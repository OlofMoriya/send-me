#!/bin/bash

data='{"text":"'$1'", "sender": "olof", "reciever":"olof", "persist": true}'

curl -d "$data" -H "api_key:testtesttest" -X POST http://127.0.0.1:8080/add
