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

# Tags & roles
echo -e "\nTesting Tags & Roles..."
validate "tags_n_roles" "policies.cedar" "policies.cedarschema"
authorize "tags_n_roles" "policies.cedar" "entities.json" "policies.cedarschema"
format "tags_n_roles" "policies.cedar"

# Discord
echo -e "\nTesting Discord..."
validate "discord" "policies.cedar" "policies.cedarschema"
authorize "discord" "policies.cedar" "entities.json" "policies.cedarschema"
format "discord" "policies.cedar"

exit "$any_failed"
