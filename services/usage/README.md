# Usage statistics service

Receives the app's daily usage report (`docs/USAGE_STATS.md` says what a report holds) and keeps
it for the questions in `queries.sql`. It runs in the maintainer's AWS account, region
ap-northeast-1 (Tokyo). Live at `https://usage.pecofence.jiang.jp/usage`.

1. The app posts its report to a CloudFront distribution. It has a name of its own because
   connections to `*.cloudfront.net` names are reset in mainland China.
2. CloudFront passes the request to the Lambda function `pecofence-usage` (`src/index.mjs`),
   which checks the report (`src/usage.mjs`) and writes it to the private bucket
   `pecofence-usage-<account id>` as `day=<day>/<id>.json`. A later upload of the same day
   replaces the earlier one.
3. Athena reads the bucket through the table `pecofence_usage.reports`.

## The request

`POST /usage` with the report as a JSON body of at most 256 KB, and the header
`x-amz-content-sha256` holding the lowercase hex SHA-256 of the body. CloudFront signs the
request for the function with that hash; without the header or with a wrong hash the answer is
`403`. The function's own URL answers `403` to everyone but the distribution.

Answers: `204` stored; `400` the report breaks a rule (`validate` in `src/usage.mjs`); `413`
over 256 KB; `405` not a POST; `404` another path; `429` more than two reports at the same
moment; `403` also for an IP address that sent more than 100 requests in 5 minutes.

```sh
body='{"v":1,"id":"5f0c2a9e7b1d4c38a6e0f1b2c3d4e5f6","day":"2026-11-14","events":[[32400,"start","by_hand"]]}'
curl -i https://usage.pecofence.jiang.jp/usage -H 'Content-Type: application/json' \
  -H "x-amz-content-sha256: $(printf %s "$body" | sha256sum | cut -d' ' -f1)" --data-binary "$body"
```

## What is kept

The reports, for 180 days, and Athena's query results, for 30 days: the bucket's lifecycle
rules delete both. No IP address is stored. The function writes the body and nothing else, and
it logs neither the body nor the id. CloudFront's access log is off, and the web ACL keeps no
samples of requests. Leave those two off.

## Deploy

`./deploy.sh`, with the AWS CLI (version 2) signed in to the account as an administrator, and
`python3` for the zip. It can be run again after a change. It creates:

- the web ACL `pecofence-usage` in us-east-1 with the rate limit (CloudFront's web ACLs live
  there, outside the stack's region);
- the stack `pecofence-usage` (`template.yaml`) in ap-northeast-1: the bucket, the function
  with its role, log group and URL, the CloudFront distribution with its origin access control,
  the subscription of the distribution to CloudFront's flat-rate Free plan, the Glue database
  and table, the Athena workgroup;
- the function's code, uploaded from `src/`.

It ends with the stack's outputs. `Endpoint` is the address for `USAGE_URL` in the app.

The Free plan costs 0 USD a month and charges nothing for traffic above its allowance
(1 million requests and 100 GB a month). The web ACL is part of it. An account can hold three
Free plans, and an account on the AWS Free Tier none. If the subscription is refused, the stack
rolls back and the web ACL is left behind, billed pay-as-you-go (about 6 USD a month). To run
without the plan: delete the stack if this was its first deployment, run `./deploy.sh payg`,
and delete the web ACL with the last two commands of "Remove everything".

Tests: `node --test` (no dependencies; the write to S3 is replaced).

### The name usage.pecofence.jiang.jp

Set up once, outside the stack. The certificate lives in us-east-1, the only region CloudFront
takes certificates from:

```sh
aws acm request-certificate --region us-east-1 --domain-name usage.pecofence.jiang.jp \
  --validation-method DNS --tags Key=project,Value=pecofence-usage
```

The zone `jiang.jp` is at Cloudflare. It holds two records for this service, both CNAME and
both DNS only (not proxied):

| Name | Target |
| --- | --- |
| `_52014062f1f567dfdf2b6c2cbbe194d4.usage.pecofence.jiang.jp` | `_a30fee4ee3b80c2f2702fe86f61832f8.wzccmgtwzk.acm-validations.aws` |
| `usage.pecofence.jiang.jp` | `dipg04t9bhh01.cloudfront.net` |

The first one proves to ACM that the name is ours and has to stay: ACM renews the certificate
through it, without anyone's help. `aws acm describe-certificate` shows it for a new
certificate. The
second one points at the distribution, the stack's output `DistributionDomain`. A stack that
was deleted and deployed again has a new one, and the record has to follow.

Once the certificate is issued, `./deploy.sh` finds it and gives the name and the certificate
to the stack (the parameters `DomainName` and `CertificateArn`). The distribution keeps
answering under its `cloudfront.net` name as well.

## Query

In the Athena console (region ap-northeast-1) choose the workgroup `pecofence-usage`, then run
a query from `queries.sql`. The workgroup sends the results to `athena-results/` in the bucket
and stops a query that would read more than 1 GB. From a shell:

```sh
id=$(aws athena start-query-execution --region ap-northeast-1 --work-group pecofence-usage \
  --query-string 'SELECT day, count(*) AS installations FROM pecofence_usage.reports GROUP BY day ORDER BY day DESC' \
  --query QueryExecutionId --output text)
aws athena get-query-results --region ap-northeast-1 --query-execution-id "$id" --output text
```

## Remove everything

```sh
aws s3 rm --recursive s3://pecofence-usage-<account id>   # the stack only deletes an empty bucket
aws cloudformation delete-stack --region ap-northeast-1 --stack-name pecofence-usage
aws cloudformation wait stack-delete-complete --region ap-northeast-1 --stack-name pecofence-usage
aws wafv2 list-web-acls --scope CLOUDFRONT --region us-east-1   # Id and LockToken of pecofence-usage
aws wafv2 delete-web-acl --scope CLOUDFRONT --region us-east-1 --name pecofence-usage --id <Id> --lock-token <LockToken>
aws acm list-certificates --region us-east-1                    # the ARN for usage.pecofence.jiang.jp
aws acm delete-certificate --region us-east-1 --certificate-arn <ARN>
```

Deleting the stack cancels the Free plan first and then waits for CloudFront to take the
distribution down, about five minutes in all. The certificate can only be deleted after the
distribution. Last, delete the two CNAME records in the Cloudflare dashboard (zone `jiang.jp`,
DNS, Records): `usage.pecofence` and `_52014062f1f567dfdf2b6c2cbbe194d4.usage.pecofence`.
