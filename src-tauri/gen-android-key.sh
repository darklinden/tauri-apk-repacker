#!/usr/bin/env bash

CWD=$(pwd)

KEY_FILE_NAME="key.keystore"
KEY_FILE_NAME_PREFIX="key"
KEY_FILE_NAME_SUFFIX=".keystore"

COUNTER=1
while [ -f "$KEY_FILE_NAME" ]; do
    KEY_FILE_NAME="${KEY_FILE_NAME_PREFIX}_${COUNTER}${KEY_FILE_NAME_SUFFIX}"
    COUNTER=$(($COUNTER + 1))
done

keytool -genkey -v \
    -dname "cn=Unknown, ou=Unknown, o=Unknown, c=Unknown" \
    -keystore $KEY_FILE_NAME \
    -alias key \
    -keyalg RSA \
    -keysize 2048 \
    -validity 36500 \
    -storepass 123456 \
    -keypass 123456

keytool -importkeystore -srckeystore $KEY_FILE_NAME -srcstorepass 123456 -destkeystore $KEY_FILE_NAME -deststoretype pkcs12

# echo "{\"key_path\": \"$KEY_FILE_NAME\", \"alias_name\": \"key\", \"store_pwd\": \"123456\", \"key_pwd\": \"123456\"}" >key.json

echo "$KEY_FILE_NAME Done!"
