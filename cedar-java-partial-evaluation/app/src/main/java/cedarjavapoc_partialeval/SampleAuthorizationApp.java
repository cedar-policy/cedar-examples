/*
 * Copyright 2022-2023 Amazon.com, Inc. or its affiliates. All Rights Reserved.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *      https://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */
package cedarjavapoc_partialeval;

import com.cedarpolicy.AuthorizationEngine;
import com.cedarpolicy.BasicAuthorizationEngine;
import com.cedarpolicy.model.AuthorizationRequest;
import com.cedarpolicy.model.AuthorizationResponse;
import com.cedarpolicy.model.AuthorizationSuccessResponse;
import com.cedarpolicy.model.PartialAuthorizationRequest;
import com.cedarpolicy.model.PartialAuthorizationResponse;
import com.cedarpolicy.model.PartialAuthorizationSuccessResponse;
import com.cedarpolicy.model.entity.Entity;
import com.cedarpolicy.model.exception.AuthException;
import com.cedarpolicy.model.exception.InternalException;
import com.cedarpolicy.model.policy.PolicySet;
import com.cedarpolicy.model.schema.Schema;
import com.cedarpolicy.model.schema.Schema.JsonOrCedar;
import com.cedarpolicy.value.EntityUID;
import com.cedarpolicy.value.PrimBool;
import com.cedarpolicy.value.Value;
import com.fasterxml.jackson.core.JsonProcessingException;
import com.fasterxml.jackson.databind.JsonNode;
import com.fasterxml.jackson.databind.ObjectMapper;
import com.fasterxml.jackson.databind.node.JsonNodeFactory;
import com.fasterxml.jackson.databind.node.ObjectNode;

import java.io.File;
import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.HashMap;
import java.util.HashSet;
import java.util.List;
import java.util.Map;
import java.util.NoSuchElementException;
import java.util.Optional;
import java.util.Set;

/**
 * A sample application demonstrating Cedar authorization functionality.
 * 
 * This class provides examples of using the Cedar authorization engine to:
 * - Perform simple authorization checks
 * - Execute partial evaluations to determine potentially accessible resources
 * - Load and parse Cedar policies and schemas
 * - Build and manage entity relationships
 */
public class SampleAuthorizationApp {

        private AuthorizationEngine authorizationEngine;
        private PolicySet policySet;
        private Set<Entity> entities;
        private Schema schema;

        /**
         * Initializes a new SampleApp instance with the specified policy set and schema
         * files.
         * 
         * This constructor sets up the Cedar authorization engine and loads the
         * required components:
         * - Creates a new BasicAuthorizationEngine instance
         * - Loads and parses the policy set from the specified file
         * - Builds the entity set containing principals, resources and actions
         * - Loads and parses the Cedar schema from the specified file
         *
         * @param policySetFilePath Path to the file containing Cedar policies to load
         * @param schemaFilePath    Path to the file containing the Cedar schema to load
         * @throws IOException       If there is an error reading either the policy set
         *                           or schema files
         * @throws InternalException If there is an error parsing the policy set or
         *                           schema files
         */
        public SampleAuthorizationApp(String policySetFilePath, String schemaFilePath)
                        throws IOException, InternalException {
                this.authorizationEngine = new BasicAuthorizationEngine();
                this.policySet = loadPolicySetFromFile(policySetFilePath);
                this.entities = buildEntities();
                this.schema = loadCedarSchemaFromFile(schemaFilePath);
        }

        /**
         * Performs a simple authorization check using the Cedar authorization engine.
         * 
         *
         * @throws InternalException If there is an internal error in the Cedar engine
         * @throws IOException       If there is an error reading required files
         * @throws AuthException     If there is an authorization-related error
         */
        public AuthorizationSuccessResponse simpleAuthorization(String principalString, String actionString, String resourceString, Map<String, Value> context) throws InternalException, IOException, AuthException, NoSuchElementException {
                System.out.println("==Complete Authorization Example==");
                // Request
                Entity prinicipal = new Entity(EntityUID.parse(principalString).get(), new HashMap<>(),
                                new HashSet<>());
                Entity action = new Entity(EntityUID.parse(actionString).get(), new HashMap<>(),
                                new HashSet<>());
                Entity resource = new Entity(EntityUID.parse(resourceString).get(), new HashMap<>(),
                                new HashSet<>());

                // isAuthorized Request with schema and schema validation
                AuthorizationRequest request = new AuthorizationRequest(prinicipal, action, resource,
                                Optional.of(context),
                                Optional.of(schema), true);
                System.out.println(request);
                AuthorizationResponse authorizationResponse = this.authorizationEngine.isAuthorized(request,
                                this.policySet,
                                this.entities);
                
                if (authorizationResponse.type == AuthorizationResponse.SuccessOrFailure.Success) {
                        System.err.printf("Decision: %s%n", authorizationResponse.success.get());
                        return authorizationResponse.success.get();
                } else {
                        System.out.print("Errors: ");
                        authorizationResponse.errors.orElseThrow()
                                        .stream()
                                        .map(error -> error.message)
                                        .forEach(System.out::println);
                        return null;
                }
        }

        /**
         * Performs partial evaluation to identify potentially accessible resources.
         * 
         * Given a principal, action and context, this method evaluates which resources
         * the
         * principal may be authorized to access. The evaluation is "partial" because
         * the
         * resource is left unknown.
         * 
         * The method:
         * 1. Creates entities from the provided principal and action strings
         * 2. Builds a partial authorization request with the known values
         * 3. Executes the partial evaluation
         * 4. Processes and returns the policies that may determine access
         * 5. The function also pretty-prints the results for demonstration purposes
         *
         * @param principalString The principal entity ID (e.g. "User::\"Alice_admin\"")
         * @param ActionString    The action entity ID (e.g. "Action::\"Write\"")
         * @param context         Map of context values for the authorization request
         * @return Set of JsonNodes containing the potentially determining policies
         * @throws InternalException      If there is an internal Cedar engine error
         * @throws IOException            If there is an error reading files
         * @throws AuthException          If there is an authorization error
         * @throws NoSuchElementException If required response elements are missing
         */
        public Set<JsonNode> partialEvaluation(String principalString, String ActionString, Map<String, Value> context)
                        throws InternalException, IOException, AuthException, NoSuchElementException {
                System.out.println("====Partial Evaluation Example====");

                // Change principal, action or add a resource entity here
                Entity prinicipal = new Entity(EntityUID.parse(principalString).get(), new HashMap<>(),
                                new HashSet<>());
                Entity action = new Entity(EntityUID.parse(ActionString).get(), new HashMap<>(),
                                new HashSet<>());

                System.out.printf("Partial Request: Principal = %s, Action = %s, Resource = Unknown, Context = %s%n",
                                prinicipal.getEUID(), action.getEUID(), context);
                // Partial Evaluation Request
                // Use the builder pattern to define known attributes. Not providing attributes
                // in the builder treats them as unknown.
                // In this example, we provide the principal, action and context but leave the
                // resource unknown
                // This allows us to evaluate what resources the principal MAY BE DETERMINING to
                // access given the specified action and context
                PartialAuthorizationRequest request = PartialAuthorizationRequest.builder()
                                .principal(prinicipal.getEUID())
                                .action(action.getEUID())
                                .context(context)
                                .schema(schema)
                                .enableRequestValidation()
                                .build();

                PartialAuthorizationResponse partialAuthorizationResponse = authorizationEngine.isAuthorizedPartial(
                                request,
                                this.policySet, this.entities);

                Set<JsonNode> maybeDeterminingPolicies = processPartialEvalResponse(partialAuthorizationResponse);

                ObjectMapper mapper = new ObjectMapper();
                // Pretty-print policies as formatted JSON with error handling for readability
                System.out.println(
                                "Following policies found to be `Maybe Determining` by Partial Evaluation: ");
                try {
                        String prettyPolicies = mapper.writerWithDefaultPrettyPrinter()
                                        .writeValueAsString(maybeDeterminingPolicies);
                        System.out.println(prettyPolicies);
                } catch (JsonProcessingException e) {
                        System.out.println(maybeDeterminingPolicies);
                }
                return maybeDeterminingPolicies;
        }

        /**
         * Processes the response from a partial authorization evaluation.
         * 
         * Takes a PartialAuthorizationResponse and extracts information about policies
         * that may
         * determine authorization decisions. For successful evaluations, it:
         * 
         * 1. Gets the residuals and "maybe determining" policies from the response
         * 2. For each policy that may be determining:
         * - Creates a JSON node with the policy ID
         * - Adds the policy effect (permit/deny) from residuals
         * - Adds any relevant entities found in residuals
         * 3. Returns the set of policy JSON nodes
         * 
         * For failed evaluations, prints error messages from the response.
         *
         * @param partialAuthorizationResponse Response from partial authorization
         *                                     containing
         *                                     either success details or error messages
         * @return Set of JsonNodes containing details about maybe determining policies
         * @throws NoSuchElementException if required response elements are missing
         */
        private Set<JsonNode> processPartialEvalResponse(PartialAuthorizationResponse partialAuthorizationResponse)
                        throws NoSuchElementException {
                Set<JsonNode> maybeDeterminingPolicies = new HashSet<>();
                if (partialAuthorizationResponse.type == PartialAuthorizationResponse.SuccessOrFailure.Success) {
                        PartialAuthorizationSuccessResponse successResponse = partialAuthorizationResponse.success
                                        .get();
                        Map<String, JsonNode> allResiduals = successResponse.getResiduals();
                        Set<String> mayBeDeterminingPolicyID = successResponse.getMayBeDetermining();

                        mayBeDeterminingPolicyID.forEach(policyId -> {
                                ObjectNode policyNode = JsonNodeFactory.instance.objectNode();
                                policyNode.set("PolicyID", JsonNodeFactory.instance.pojoNode(policyId));
                                policyNode.set("PolicyEffect", findEffectFromResiduals(allResiduals.get(policyId)));
                                List<JsonNode> residualEntities = findEntitiesFromResiduals(allResiduals.get(policyId));
                                var entitiesValue = residualEntities.isEmpty()
                                                ? "No unknown entities found OR policy covers all unknown entities"
                                                : residualEntities;
                                policyNode.set("Relevant_Entities",
                                                JsonNodeFactory.instance.pojoNode(entitiesValue));
                                maybeDeterminingPolicies.add(policyNode);
                        });

                } else {
                        System.out.print("Errors: ");
                        partialAuthorizationResponse.errors.get()
                                        .stream()
                                        .map(error -> error.message)
                                        .forEach(System.out::println);
                }
                return maybeDeterminingPolicies;
        }

        /**
         * Creates a sample entity set containing Document resources, User principals
         * and Actions.
         * The entities are organized into groups:
         * - DocumentGroup::Public containing public_doc1 and public_doc2
         * - DocumentGroup::Confidential containing confidential_doc1 and
         * confidential_doc2
         * - UserGroup::Admin containing Alice_admin and Bob_admin
         * - UserGroup::Customer containing John_customer and Mark_customer
         * - Actions: Read, Write, Delete
         * 
         * These entities correspond to the ones in
         * $PROJECTDIR/src/main/resources/entities.json.
         * Currently, CedarJava does not provide a direct capability to load entities
         * from JSON,
         * so we create them programmatically in this method.
         * 
         * @return Set<Entity> containing the sample entities and their relationships
         */
        private Set<Entity> buildEntities() {

                Set<Entity> entities = new HashSet<>();

                // Add Document Resource Entities for DocumentGroup::Public
                Entity publicDocumentGroup = new Entity(EntityUID.parse("DocumentGroup::\"Public\"").get(),
                                new HashMap<>(),
                                new HashSet<>());
                Set<EntityUID> publicDocumentGroupSet = new HashSet<>();
                publicDocumentGroupSet.add(publicDocumentGroup.getEUID());
                entities.add(publicDocumentGroup);
                entities.add(new Entity(EntityUID.parse("Document::\"public_doc1\"").get(), new HashMap<>(),
                                publicDocumentGroupSet));
                entities.add(new Entity(EntityUID.parse("Document::\"public_doc2\"").get(), new HashMap<>(),
                                publicDocumentGroupSet));
                entities.add(new Entity(EntityUID.parse("Document::\"public_doc3\"").get(), new HashMap<>(),
                                publicDocumentGroupSet));

                // Add Document Resource Entities for DocumentGroup::Confidential
                Entity confidentialDocumentGroup = new Entity(EntityUID.parse("DocumentGroup::\"Confidential\"").get(),
                                new HashMap<>(), new HashSet<>());
                Set<EntityUID> confidentialDocumentGroupSet = new HashSet<>();
                confidentialDocumentGroupSet.add(confidentialDocumentGroup.getEUID());
                entities.add(confidentialDocumentGroup);
                entities.add(new Entity(EntityUID.parse("Document::\"confidential_doc1\"").get(), new HashMap<>(),
                                confidentialDocumentGroupSet));
                entities.add(new Entity(EntityUID.parse("Document::\"confidential_doc2\"").get(), new HashMap<>(),
                                confidentialDocumentGroupSet));
                entities.add(new Entity(EntityUID.parse("Document::\"confidential_doc3\"").get(), new HashMap<>(),
                                confidentialDocumentGroupSet));

                // Add Document Resource Entities for DocumentGroup::Protected
                Entity protectedDocumentGroup = new Entity(EntityUID.parse("DocumentGroup::\"Protected\"").get(),
                                new HashMap<>(), new HashSet<>());
                Set<EntityUID> protectedDocumentGroupSet = new HashSet<>();
                protectedDocumentGroupSet.add(protectedDocumentGroup.getEUID());
                entities.add(protectedDocumentGroup);
                entities.add(new Entity(EntityUID.parse("Document::\"protected_doc1\"").get(), new HashMap<>(),
                                protectedDocumentGroupSet));
                entities.add(new Entity(EntityUID.parse("Document::\"protected_doc2\"").get(), new HashMap<>(),
                                protectedDocumentGroupSet));
                entities.add(new Entity(EntityUID.parse("Document::\"protected_doc3\"").get(), new HashMap<>(),
                                protectedDocumentGroupSet));

                // Add User Principal Entities for UserGroup::Admin
                Entity adminUserGroup = new Entity(EntityUID.parse("UserGroup::\"Admin\"").get(), new HashMap<>(),
                                new HashSet<>());
                Set<EntityUID> adminUserGroupSet = new HashSet<>();
                adminUserGroupSet.add(adminUserGroup.getEUID());
                entities.add(adminUserGroup);
                entities.add(new Entity(EntityUID.parse("User::\"Alice_admin\"").get(), new HashMap<>(),
                                adminUserGroupSet));
                entities.add(new Entity(EntityUID.parse("User::\"Bob_admin\"").get(), new HashMap<>(),
                                adminUserGroupSet));

                // Add User Principal Entities for UserGroup::Customer
                Entity customerUserGroup = new Entity(EntityUID.parse("UserGroup::\"Customer\"").get(), new HashMap<>(),
                                new HashSet<>());
                Set<EntityUID> customerUserGroupSet = new HashSet<>();
                customerUserGroupSet.add(customerUserGroup.getEUID());
                entities.add(customerUserGroup);
                entities.add(new Entity(EntityUID.parse("User::\"John_customer\"").get(), new HashMap<>(),
                                customerUserGroupSet));
                entities.add(new Entity(EntityUID.parse("User::\"Mark_customer\"").get(), new HashMap<>(),
                                customerUserGroupSet));

                // Add Actions
                entities.add(new Entity(EntityUID.parse("Action::\"Read\"").get(), new HashMap<>(), new HashSet<>()));
                entities.add(new Entity(EntityUID.parse("Action::\"Write\"").get(), new HashMap<>(), new HashSet<>()));
                entities.add(new Entity(EntityUID.parse("Action::\"Delete\"").get(), new HashMap<>(), new HashSet<>()));

                return entities;
        }

        /**
         * Loads a PolicySet from a file on disk. A PolicySet represents a collection of
         * Cedar
         * authorization policies that can be evaluated by the Cedar authorization
         * engine.
         * 
         * The file should contain valid Cedar policy syntax. For example:
         * permit(principal == User::"alice", action == Action::"view", resource ==
         * Photo::"pic01");
         *
         * @param filepath The path to the policy file to load. Must point to a readable
         *                 file containing Cedar policies.
         * @return PolicySet containing the parsed Cedar policies loaded from the file
         * @throws InternalException If there is an error parsing the policy file
         *                           contents as valid Cedar policies
         * @throws IOException       If there is an error reading the file from disk
         */
        private PolicySet loadPolicySetFromFile(String filepath) throws InternalException, IOException {
                Path policiesFile = new File(filepath).toPath();
                PolicySet policySet = PolicySet.parsePolicies(policiesFile);
                return policySet;
        }

        /**
         * Loads a Cedar schema from a file on disk.
         * 
         * @param filepath The path to the schema file to load
         * @return Schema object containing the parsed Cedar schema
         * @throws InternalException If there is an error parsing the schema file
         * @throws IOException       If there is an error reading the file
         */
        private Schema loadCedarSchemaFromFile(String filepath) throws InternalException, IOException {
                Path schemaFile = new File(filepath).toPath();
                String schemaStr = Files.readString(schemaFile);
                Schema schema = Schema.parse(JsonOrCedar.Cedar, schemaStr);
                return schema;
        }

        /**
         * Extracts entity nodes from a residual JsonNode.
         * 
         * Entity nodes of residuals represent entities that were not known during
         * partial authorization
         * but are present in the policy and identified from the policy residual.
         * 
         * For example, given the policy:
         * permit(principal == User::"bob", action == Action::"view", resource ==
         * Resource::"doc1")
         * 
         * And a partial authorization request:
         * Principal = User::"bob"
         * Action = Action::"view"
         * Resource = unknown
         * 
         * This function will extract resource = Resource::"doc1" from the policy's
         * residual.
         * 
         * @param residual The residual policy JsonNode to search for entities
         * @return List of JsonNodes containing entity values, or null if no entities
         *         found
         * @throws NullPointerException if the residual parameter is null
         */
        private List<JsonNode> findEntitiesFromResiduals(JsonNode residual) throws NullPointerException {
                if (residual == null) {
                        throw new NullPointerException("Null residual");
                }
                return residual.findValues("__entity");
        }

        /**
         * Extracts the effect value from a residual JsonNode.
         * 
         * The effect value in a residual indicates whether the policy would permit or
         * deny
         * access if all unknown values were resolved. This is useful for understanding
         * the
         * potential outcome of a partial authorization request.
         * 
         * For example, given a policy:
         * permit(principal == User::"alice", action == Action::"view", resource ==
         * Resource::"doc1")
         * 
         * And a partial authorization request with unknown resource:
         * Principal = User::"alice"
         * Action = Action::"view"
         * Resource = unknown
         * 
         * This function will extract "permit" as the effect from the policy's residual.
         *
         * @param residual The residual policy JsonNode to extract the effect from
         * @return String containing the effect value ("permit" or "deny"), or null if
         *         not found
         * @throws NullPointerException if the residual parameter is null
         */
        private JsonNode findEffectFromResiduals(JsonNode residual) throws NullPointerException {
                if (residual == null) {
                        throw new NullPointerException("Null residual");
                }
                return residual.findValue("effect");
        }
}