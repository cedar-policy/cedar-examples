#!/bin/bash

failed() {
    message=$1
    echo "$message"
    exit 1
}

validate() {
    local folder=$1
    local policies=$2
    local schema=$3
    echo "Running validation on ${policies}"
    (
     res=$(cedar validate --policies $folder/$policies --schema $folder/$schema)
     if [[ $res != "Validation Passed" ]]
     then
         failed "FAIL: validate on ${policies} with result: ${res}"
     else
         echo "PASS: validate succeeded"
     fi
    )
}

authorize() {
    local folder=$1
    local policies=$2
    local entities=$3  
    echo "Running authorization on ${policies}"
    for decision in ALLOW DENY
    do
      for file in $folder/$decision/*.json
      do
          res=$(cedar authorize --policies $folder/$policies  --entities $folder/$entities --request-json "$file" | xargs)
          if [ "$res" != "$decision" ]
          then
              failed "FAIL: Authorization decision \"${res}\" on ${file}, expected \"${decision}\""
          else
              echo "PASS: Authorization decision \"${decision}\" on ${file}"
          fi
      done
    done
}

# PhotoApp
echo "Testing PhotoApp..."
validate "PhotoApp" "photoapp.cedar" "photoapp.cedarschema.json"
authorize "PhotoApp" "photoapp.cedar" "photoapp.cedarentities.json"

# GitApp
echo "Testing GitApp..."
validate "GitApp" "gitapp.cedar" "gitapp.cedarschema.json"
authorize "GitApp" "gitapp.cedar" "gitapp.cedarentities.json"