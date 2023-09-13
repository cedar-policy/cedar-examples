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

package samplelib;

import com.cedarpolicy.BasicAuthorizationEngine;
import com.cedarpolicy.AuthorizationEngine;
import com.cedarpolicy.model.AuthorizationRequest;
import com.cedarpolicy.model.AuthorizationResponse;
import com.cedarpolicy.model.slice.Slice;
import com.cedarpolicy.model.slice.BasicSlice;
import com.cedarpolicy.model.slice.Policy;
import com.cedarpolicy.model.slice.Entity;
import com.cedarpolicy.model.exception.AuthException;
import com.cedarpolicy.serializer.JsonEUID;
import java.util.HashMap;
import java.util.HashSet;
import java.util.Set;

public class SampleRunner {
    /**
     * Returns the set of policies the evaluation engine will see.
     * In this case, we have one policy, that says:
     * the principal Alice, can perform the action View_Photo, on any resource that's a child of resource Vacation
     */
    private static Set<Policy> buildPolicySlice() {
        Set<Policy> ps = new HashSet<>();
        String fullPolicy =
            "permit(principal == User::\"Alice\", action == Action::\"View_Photo\", resource in Album::\"Vacation\");";
        ps.add(new Policy(fullPolicy, "p1"));
        return ps;
    }

    private static Set<Entity> buildEntitySlice() {
        Set<Entity> e = new HashSet<>();
        Entity album = new Entity(new JsonEUID("Album", "Vacation"), new HashMap<>(), new HashSet<>());
        e.add(album);
        e.add(new Entity(new JsonEUID("User", "Alice"), new HashMap<>(), new HashSet<>()));
        e.add(new Entity(new JsonEUID("Action", "View_Photo"), new HashMap<>(), new HashSet<>()));
        Set<JsonEUID> parents = new HashSet<>();
        parents.add(album.getEUID());
        Entity photo = new Entity(new JsonEUID("Photo","pic01"), new HashMap<>(), parents);
        e.add(photo);
        return e;
    }


    private static boolean testRun(BasicSlice slice, AuthorizationEngine engine) {
        AuthorizationRequest r = new AuthorizationRequest("User::\"Alice\"",
            "Action::\"View_Photo\"",
        "Photo::\"pic01\"", new HashMap<>());
        try {
            AuthorizationResponse resp = engine.isAuthorized(r, slice);
            return resp.isAllowed();
        } catch (Exception e) {
            System.err.println("Error!");
            return false;
        }

    }

    public static void main(String[] args) {

        int NUM_PRE_RUNS = 100;
        int NUM_RUNS = 100;

        boolean res = true;

        AuthorizationEngine engine = new BasicAuthorizationEngine();
        BasicSlice slice = new BasicSlice(buildPolicySlice(), buildEntitySlice());


        for(int i = 0; i < NUM_PRE_RUNS; i++) {
            res &= testRun(slice, engine);
        }

        long start = System.currentTimeMillis();

        for(int i = 0; i < NUM_RUNS; i++) {
            res &= testRun(slice, engine);
        }

        // do something
        long end = System.currentTimeMillis();
        long executionTime = end - start;
        System.out.println("execution time: "+executionTime+" ms");

        System.out.println("res: "+res);
    }
}