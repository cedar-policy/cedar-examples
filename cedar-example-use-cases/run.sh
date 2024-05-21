#!/bin/bash

source ../test_utils.sh

# GitHub
echo -e "\nTesting github_example..."
validate "github_example" "policies.cedar" "policies.cedarschema"
authorize "github_example" "policies.cedar" "entities.json"
format "github_example" "policies.cedar"

# Document cloud
echo -e "\nTesting document_cloud..."
validate "document_cloud" "policies.cedar" "policies.cedarschema"
authorize "document_cloud" "policies.cedar" "entities.json"
format "document_cloud" "policies.cedar"

exit "$any_failed"
