#!/bin/bash

id="8dba0b7c-bb64-440d-a560-f5decef0dd54"
endpoint="http://192.168.1.39:42069/download/" 

curl "${endpoint}${id}" > out.json

python parse.py
