// The Lambda function's entry point: hands each request to `receive` (usage.mjs) together with
// the one call to AWS, the write to the bucket. The tests pass their own function instead.
//
// The SDK comes with the Lambda runtime. It is loaded here, while the function starts: loaded
// during the first request it took four seconds.

import { PutObjectCommand, S3Client } from '@aws-sdk/client-s3';
import { receive } from './usage.mjs';

const s3 = new S3Client();

const put = (key, body) =>
  s3.send(new PutObjectCommand({ Bucket: process.env.BUCKET, Key: key, Body: body, ContentType: 'application/json' }));

export const handler = (request) => receive(request, put);
