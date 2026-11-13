#!/usr/bin/env bash
# Deploys the receiving side of the usage statistics to the AWS account of the current
# credentials: the web ACL, the stack of template.yaml and the function's code. It can be run
# again after a change.
#
#   ./deploy.sh         on CloudFront's flat-rate Free plan (0 USD a month)
#   ./deploy.sh payg    without the web ACL and the plan (pay-as-you-go), for a first deployment
#                       to an account the plan refuses
set -euo pipefail
cd "$(dirname "$0")"
export AWS_PAGER=""
name=pecofence-usage
region=ap-northeast-1

# The Free plan needs a web ACL on the distribution, and CloudFront's web ACLs live in
# us-east-1, outside the stack's region. Its one rule blocks an IP address that sent more than
# 100 requests in the last 5 minutes. WAF keeps no samples of the requests (they hold IP
# addresses). Until the stack has subscribed the distribution to the plan, the web ACL is
# billed pay-as-you-go (about 6 USD a month): if the subscription is refused, delete it
# (README.md).
acl=
if [ "${1:-}" != payg ]; then
  acl=$(aws wafv2 list-web-acls --scope CLOUDFRONT --region us-east-1 \
    --query "WebACLs[?Name=='$name'].ARN | [0]" --output text)
  if [ "$acl" = None ]; then
    acl=$(aws wafv2 create-web-acl --name $name --scope CLOUDFRONT --region us-east-1 \
      --default-action 'Allow={}' \
      --visibility-config "SampledRequestsEnabled=false,CloudWatchMetricsEnabled=true,MetricName=$name" \
      --rules '[{
        "Name": "rate-limit",
        "Priority": 0,
        "Action": { "Block": {} },
        "Statement": { "RateBasedStatement": { "Limit": 100, "EvaluationWindowSec": 300, "AggregateKeyType": "IP" } },
        "VisibilityConfig": { "SampledRequestsEnabled": false, "CloudWatchMetricsEnabled": true, "MetricName": "'$name'-rate-limit" }
      }]' \
      --tags "Key=project,Value=$name" --query Summary.ARN --output text)
  fi
fi

aws cloudformation deploy --region $region --stack-name $name --template-file template.yaml \
  --capabilities CAPABILITY_NAMED_IAM --parameter-overrides "WebAclArn=$acl" \
  --tags "project=$name" --no-fail-on-empty-changeset

# The function's code: the template creates the function with a placeholder.
zip=${TMPDIR:-/tmp}/$name.zip
python3 -m zipfile -c "$zip" src/index.mjs
aws lambda update-function-code --region $region --function-name $name \
  --zip-file "fileb://$zip" --query CodeSha256 --output text
aws lambda wait function-updated --region $region --function-name $name

aws cloudformation describe-stacks --region $region --stack-name $name \
  --query 'Stacks[0].Outputs[].[OutputKey,OutputValue]' --output text
