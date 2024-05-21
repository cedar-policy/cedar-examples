#!/bin/bash

source ../test_utils.sh

# PhotoApp
echo -e "\nTesting github_example..."
validate "github_example" "policies.cedar" "github_example.cedarschema"
authorize "github_example" "policies.cedar" "entities.json"
format "github_example" "policies.cedar"

# GitApp
echo -e "\nTesting document_cloud..."
validate "document_cloud" "policies.cedar" "document_cloud.cedarschema"
authorize "document_cloud" "policies.cedar" "entities.json"
format "document_cloud" "policies.cedar"

# FIS
echo -e "\nTesting Tags & Roles..."
validate "tags_n_roles" "policies.cedar" "policies.cedarschema"
authorize "tags_n_roles" "policies.cedar" "entities.json" "policies.cedarschema"
format "tags_n_roles" "policies.cedar"

exit "$any_failed"
