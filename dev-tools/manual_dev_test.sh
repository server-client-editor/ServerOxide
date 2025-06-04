#!/usr/bin/bash -x

HOST=127.0.0.1:8443
API_BASE_URL=https://$HOST/api/v1
WS_BASE_URL=wss://$HOST/api/v1

CERT_PATH=../certs/dev_cert.pem

curl "$API_BASE_URL/captcha" --cacert "$CERT_PATH"
curl "$API_BASE_URL/login" --cacert "$CERT_PATH" -H "Content-Type: application/json" -d '{"key":"value"}'
curl "$API_BASE_URL/signup" --cacert "$CERT_PATH" -H "Content-Type: application/json" -d '{"key":"value"}'
wscat -c "$WS_BASE_URL/chat" --no-check
