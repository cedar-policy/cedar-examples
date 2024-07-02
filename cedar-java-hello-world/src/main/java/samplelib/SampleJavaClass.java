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
import com.cedarpolicy.value.EntityTypeName;
import com.cedarpolicy.value.EntityUID;
import java.util.HashMap;
import java.util.HashSet;
import java.util.Set;

/**
 * Documentation for SampleJavaClass.
 */
public class SampleJavaClass {

    final EntityTypeName principalType, actionType, albumResourceType, photoResourceType;

    public SampleJavaClass() {
        principalType = EntityTypeName.parse("User").get();
        actionType = EntityTypeName.parse("Action").get();
        albumResourceType = EntityTypeName.parse("Album").get();
        photoResourceType = EntityTypeName.parse("Photo").get();
    }

    /**
     * Execute the query "Can principal Alice perform the action View_Photo on resource Pic01".
     */
    public boolean sampleMethod() throws AuthException {
        Entity principal = new Entity(principalType.of("Alice"), new HashMap<>(), new HashSet<>());
        Entity action = new Entity(actionType.of("View_Photo"), new HashMap<>(), new HashSet<>());
        Entity resource = new Entity(photoResourceType.of("pic01"), new HashMap<>(), new HashSet<>());

        AuthorizationEngine ae = new BasicAuthorizationEngine();
        AuthorizationRequest r = new AuthorizationRequest(principal.getEUID(), action.getEUID(), resource.getEUID(), new HashMap<>());
        return ae.isAuthorized(r, buildSlice()).isAllowed();
    }

    /**
     * Build the slice of the store the cedar evaluator will see.
     */
    private Slice buildSlice() {
        Set<Policy> p = buildPolicySlice();
        Set<Entity> e = buildEntitySlice();
        return new BasicSlice(p, e);
    }

    /**
     * Returns the set of policies the evaluation engine will see.
     * In this case, we have one policy, that says:
     * the principal Alice, can perform the action View_Photo, on any resource that's a child of resource Vacation
     */
    private Set<Policy> buildPolicySlice() {
        Set<Policy> ps = new HashSet<>();
        String fullPolicy =
            "permit(principal == User::\"Alice\", action == Action::\"View_Photo\", resource in Album::\"Vacation\");";
        ps.add(new Policy(fullPolicy, "p1"));
        return ps;
    }

    /**
     * Create the set of entities the evaluation engine will see.
     * In this case we have one user Alice
     * One action View_Photo
     * A resource Vacation that has two children, pic01 and pic02
     */
    private Set<Entity> buildEntitySlice() {
        Set<Entity> e = new HashSet<>();
        Entity album = new Entity(albumResourceType.of("Vacation"), new HashMap<>(), new HashSet<>());
        e.add(album);
        e.add(new Entity(principalType.of("Alice"), new HashMap<>(), new HashSet<>()));
        e.add(new Entity(actionType.of("View_Photo"), new HashMap<>(), new HashSet<>()));
        Set<EntityUID> parents = new HashSet<>();
        parents.add(album.getEUID());
        Entity photo = new Entity(photoResourceType.of("pic01"), new HashMap<>(), parents);
        e.add(photo);
        return e;
    }

    /**
     * Execute a query with an invalid policy to show errors.
     */
    public AuthorizationResponse shouldFail() throws AuthException {
        Entity principal = new Entity(principalType.of("Alice"), new HashMap<>(), new HashSet<>());
        Entity action = new Entity(actionType.of("View_Photo"), new HashMap<>(), new HashSet<>());
        Entity resource = new Entity(photoResourceType.of("pic01"), new HashMap<>(), new HashSet<>());

        AuthorizationEngine ae = new BasicAuthorizationEngine();
        AuthorizationRequest r = new AuthorizationRequest(principal.getEUID(), action.getEUID(), resource.getEUID(), new HashMap<>());
        AuthorizationResponse resp = ae.isAuthorized(r, buildFailingSlice());
        return resp;
    }

    /**
     * Build a slice that contains an invalid policy
     */
    private Slice buildFailingSlice() {
        Set<Policy> p = buildUnparseable();
        Set<Entity> e = buildEntitySlice();
        return new BasicSlice(p, e);
    }


    /**
     * Returns a set containing a non-gramatically correct policy
     */
    private Set<Policy> buildUnparseable() {
        Set<Policy> ps = new HashSet<>();
        ps.add(new Policy("not a policy", "p2"));
        return ps;
    }

}
