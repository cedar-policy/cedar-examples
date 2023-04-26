#!/bin/bash

#Change depending on where your have your CedarCLI executable. e.g., 
#export CEDAR_BIN_DIR="~/workspace/cedar/CedarCLI/build/bin/"
#alternatively, just add the cedar executable to your path
if [[ -z "${CEDAR_BIN_DIR}" ]];
then
    :
else
    export PATH=$PATH:$CEDAR_BIN_DIR
fi


#validate policies
res=$(cedar validate --policies policies.cedar --schema schema.json)
if [[ $res != "Validation Passed" ]]
then
    echo "Failed Validation"
else
    echo "Validation succeeded"
fi


#track so we can print success if applicable
ANY_TEST_FAILED=false

for file in ./allow_queries/*
do
res=$(cedar authorize --policies policies.cedar --entities entities.json --query-json $file)
if [ $res != "ALLOW" ]
then
    echo "Failed on " $file " with result " $res
    ANY_TEST_FAILED=true
fi
done

for file in ./deny_queries/*
do
res=$(cedar authorize --policies policies.cedar --entities entities.json --query-json $file)
if [ $res != "DENY" ]
then
    echo "Failed on " $file " with result " $res
    ANY_TEST_FAILED=true
fi
done

if $ANY_TEST_FAILED
then
    echo "Not all tests succeeded"
else
    echo "All tests finished successfully"
fi