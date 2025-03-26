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

import org.apache.log4j.Logger;
import org.apache.log4j.Level;
import java.util.HashMap;
import java.util.Map;

import com.cedarpolicy.value.Value;

import org.apache.log4j.BasicConfigurator;

public class Launcher {
    public static void main(String[] args) {
        BasicConfigurator.configure();
        Logger.getRootLogger().setLevel(Level.WARN);
        String policySetFilePath = "./src/main/resources/policies.cedar";
        String schemaFilePath = "./src/main/resources/sampleapp.cedarschema";
        try {
            Map<String, Value> context = new HashMap<>();
            // context.put("authenticated", new PrimBool(true)); // Uncomment for delete or write action. Comment for read action
            // context.put("delete_confirmation", new PrimBool(true)); // Uncomment for delete action. Comment for read or write action

            String principalString = "User::\"Alice_admin\"";
            String actionString = "Action::\"Read\"";
            String resourceString = "Document::\"protected_doc1\"";

            SampleAuthorizationApp sampleApp = new SampleAuthorizationApp(policySetFilePath, schemaFilePath);

            // Partial Evalation Example
            sampleApp.partialEvaluation(principalString, actionString, context);

            // Complete Authorization Example
            sampleApp.simpleAuthorization(principalString, actionString, resourceString, context);

        } catch (Exception e) {
            e.printStackTrace();
        }

    }
}
