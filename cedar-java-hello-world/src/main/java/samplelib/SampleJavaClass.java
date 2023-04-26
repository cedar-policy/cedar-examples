package samplelib;

import cedarpolicy.WrapperAuthorizationEngine;
import cedarpolicy.AuthorizationEngine;
import cedarpolicy.model.AuthorizationQuery;
import cedarpolicy.model.AuthorizationResult;
import cedarpolicy.model.slice.Slice;
import cedarpolicy.model.slice.BasicSlice;
import cedarpolicy.model.slice.Policy;
import cedarpolicy.model.slice.Entity;
import cedarpolicy.model.exception.AuthException;
import java.util.HashMap;
import java.util.HashSet;
import java.util.Set;

/**
 * Documentation for SampleJavaClass.
 */
public class SampleJavaClass {

    /**
     * Execute the query "Can principal Alice perform the action View_Photo on resource Pic01".
     */
    public boolean sampleMethod() throws AuthException {
        AuthorizationEngine ae = new WrapperAuthorizationEngine();
        AuthorizationQuery q = new AuthorizationQuery("User::\"Alice\"",
            "Action::\"View_Photo\"",
        "Photo::\"pic01\"");
        return ae.isAuthorized(q, buildSlice()).isAllowed();
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
        Entity album = new Entity("Album::\"Vacation\"");
        e.add(album);
        e.add(new Entity("User::\"Alice\""));
        e.add(new Entity("Action::\"View_Photo\""));
        Set<String> parents = new HashSet<>();
        parents.add(album.uid);
        Entity photo = new Entity("Photo::\"pic01\"", new HashMap<>(), parents);
        e.add(photo);
        return e;
    }

    /**
     * Execute a query with an invalid policy to show errors.
     */
    public AuthorizationResult shouldFail() throws AuthException {
        AuthorizationEngine ae = new WrapperAuthorizationEngine();
        AuthorizationQuery q = new AuthorizationQuery("User::\"Alice\"",
            "Action::\"View_Photo\"",
        "Photo::\"pic01\"");
        AuthorizationResult r = ae.isAuthorized(q, buildFailingSlice());
        return r;
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
