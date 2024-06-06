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

# Sales org static
echo -e "\nTesting Sales Orgs (static)..."
validate "sales_orgs/static" "policies.cedar" "policies.cedarschema"
authorize "sales_orgs/static" "policies.cedar" "entities.json" "policies.cedarschema"
#format "sales_orgs/static" "policies.cedar"

# Sales org templated
echo -e "\nTesting Sales Orgs (templated)..."
validate "sales_orgs/templated" "policies.cedar" "policies.cedarschema" "linked"
authorize "sales_orgs/templated" "policies.cedar" "entities.json" "policies.cedarschema" "linked"
#format "sales_orgs/templated" "policies.cedar"

exit "$any_failed"
