#!/usr/bin/bash -x

HOST=127.0.0.1:8443
API_BASE_URL=http://$HOST/api/v1
WS_BASE_URL=ws://$HOST/api/v1

curl "$API_BASE_URL/captcha"
curl "$API_BASE_URL/login" -H "Content-Type: application/json" -d '{"key":"value"}'
curl "$API_BASE_URL/signup" -H "Content-Type: application/json" -d '{"key":"value"}'
wscat -c "$WS_BASE_URL/chat"
