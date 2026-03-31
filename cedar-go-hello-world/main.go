package main

import (
	"fmt"

	"github.com/cedar-policy/cedar-go"
	"github.com/cedar-policy/cedar-go/ast"
)

func main() {
	// For an authorization evaluation, we need three inputs:
	// 1. A cedar.PolicySet to evaluate against the request.
	// 2. A cedar.EntityMap that contains the entities referenced in the policies and request.
	// 3. A cedar.Request to evaluate.
	// In this example, we define the policies, entities, and requests inline,
	// but they could also be loaded from external sources.
	policySet := buildPolicySet()
	entityMap := buildEntityMap()
	// We build a slice of requests to showcase the different outcomes of the cedar.Authorize method.
	evalRequests := buildRequests()

	// The first request is for Alice to view JanesVacation.jpg, which should be allowed by policy0.
	fmt.Println("Evaluating if Alice can view JanesVacation.jpg...")
	ok, diag := cedar.Authorize(policySet, entityMap, evalRequests[0])
	outputDecision(ok, diag)

	// The second request is for Alice to view JanesVacationPrivate.jpg, which should be denied by policy2.
	fmt.Println("Evaluating if Alice can view JanesVacationPrivate.jpg...")
	ok, diag = cedar.Authorize(policySet, entityMap, evalRequests[1])
	outputDecision(ok, diag)

	// The third request is for Alice to view John's vacation album, which should be denied and error
	// as the resource Album::"john_vacation" does not exist in the entity map.
	fmt.Println("Evaluating if Alice can view John's vacation album...")
	ok, diag = cedar.Authorize(policySet, entityMap, evalRequests[2])
	outputDecision(ok, diag)

	// The fourth request is for Bob to view JanesVacation.jpg, which should be denied as no policy gives permission.
	fmt.Println("Evaluating if Bob can view JanesVacation.jpg...")
	ok, diag = cedar.Authorize(policySet, entityMap, evalRequests[3])
	outputDecision(ok, diag)
}

// buildPolicySet constructs a PolicySet with three policies:
// 1. A standard permit policy defined via JSON.
// 2. A permit policy with context defined via Cedar syntax.
// 3. A forbid policy constructed via AST.
// It returns the constructed PolicySet for use in authorization evaluations.
func buildPolicySet() *cedar.PolicySet {
	// policy0 allows Alice to view the photo JanesVacation.jpg
	policy0 := buildPolicyFromJson()
	// policy1 allows Alice to view a photo if it's tagged with work
	policy1 := buildPolicyFromCedar()
	// policy2 forbids access to resources tagged as private unless the principal is the owner of the resource.
	policy2 := buildPolicyFromAST()

	policySet := cedar.NewPolicySet()
	policySet.Add("policy0", &policy0)
	policySet.Add("policy1", &policy1)
	policySet.Add("policy2", policy2)

	return policySet
}

// buildPolicyFromJson constructs a simple permit policy from a JSON representation.
// The policy allows the user "alice" to perform the "view" action on the Photo resource "JanesVacation.jpg".
func buildPolicyFromJson() cedar.Policy {
	var jsonPolicy = []byte(`{
		"effect": "permit",
		"principal": {"op": "==", "entity": {"type": "User", "id": "alice"}},
		"action": {"op": "==", "entity": {"type": "Action", "id": "view"}},
		"resource": {"op": "==", "entity": {"type": "Photo", "id": "JanesVacation.jpg"}}
    }`)

	var policy0 cedar.Policy
	if err := policy0.UnmarshalJSON(jsonPolicy); err != nil {
		fmt.Println("Unmarshal error for JSON policy:", err)
		return cedar.Policy{}
	}

	return policy0
}

// buildPolicyFromCedar constructs a permit policy from a Cedar syntax representation.
// The policy allows the user "john" to perform the "view" action on the "jane_vacation" album resource
// when the resource is tagged with "work".
func buildPolicyFromCedar() cedar.Policy {
	var cedarPolicy = []byte(`permit (
		principal == User::"john",
		action == Action::"view",
		resource in Album::"jane_vacation"
  	)
	when{ resource.tags.contains("work") };`)

	var policy1 cedar.Policy
	if err := policy1.UnmarshalCedar(cedarPolicy); err != nil {
		fmt.Println("Unmarshal error for Cedar policy:", err)
		return cedar.Policy{}
	}
	return policy1
}

// buildPolicyFromAST constructs a forbid policy programmatically using the AST package.
// The policy forbids actions when the resource is tagged as private,
// unless the principal is the owner of the resource.
func buildPolicyFromAST() *cedar.Policy {

	astPolicy := ast.Forbid().
		When(ast.Resource().Access("tags").Contains(ast.String("private"))).
		Unless(ast.Resource().Access("owner").Equal(ast.Principal()))

	policy2 := cedar.NewPolicyFromAST(astPolicy)
	return policy2
}

// buildEntityMap constructs an EntityMap containing the entities referenced in the policies and requests.
func buildEntityMap() cedar.EntityMap {
	var jsonEntities = []byte(`[
		{
			"uid": { "type": "User", "id": "alice" },
			"attrs": {},
			"parents": []
		},
		{
			"uid": { "type": "User", "id": "john" },
			"attrs": {},
			"parents": []
		},
		{
			"uid": { "type": "User", "id": "jane" },
			"attrs": {},
			"parents": []
		},
		{
			"uid": { "type": "Photo", "id": "JanesVacation.jpg" },
			"attrs": { "tags": [], "owner": "User::jane" },
			"parents": [{ "type": "Album", "id": "jane_vacation" }]
		},
		{
			"uid": { "type": "Photo", "id": "JanesVacationPrivate.jpg" },
			"attrs": { "tags": ["private"], "owner": "User::jane" },
			"parents": [{ "type": "Album", "id": "jane_vacation" }]
		},
		{
			"uid": { "type": "Album", "id": "jane_vacation" },
			"attrs": { "tags": [], "owner": "User::jane" },
			"parents": []
		}
	]`)

	var entityMap cedar.EntityMap
	if err := entityMap.UnmarshalJSON(jsonEntities); err != nil {
		fmt.Println("Entity map unmarshal error:", err)
		return cedar.EntityMap{}
	}
	return entityMap
}

// buildRequests constructs a slice of Requests to pass to the Authorize method for evaluation. Each request is designed to test different policies and outcomes:
// 1. Alice viewing JanesVacation.jpg (should be allowed by policy0).
// 2. Alice viewing JanesVacationPrivate.jpg (should be denied by policy2).
// 3. Alice viewing John's vacation album (should be denied due to an error of the resource missing from the entity map and therefore unable to fulfil policy2)
// 4. John viewing JanesVacation.jpg (should be denied because no policy applies).
func buildRequests() []cedar.Request {

	makeRequest := func(principalType, actionType, resourceType cedar.EntityType, principalID, actionID, resourceID cedar.String) cedar.Request {
		return cedar.Request{
			Principal: cedar.NewEntityUID(principalType, principalID),
			Action:    cedar.NewEntityUID(actionType, actionID),
			Resource:  cedar.NewEntityUID(resourceType, resourceID),
		}
	}

	return []cedar.Request{
		makeRequest("User", "Action", "Photo", "alice", "view", "JanesVacation.jpg"),
		makeRequest("User", "Action", "Photo", "alice", "view", "JanesVacationPrivate.jpg"),
		makeRequest("User", "Action", "Album", "alice", "view", "john_vacation"),
		makeRequest("User", "Action", "Photo", "john", "view", "JanesVacation.jpg"),
	}
}

// outputDecision is a helper function that takes an authorization decision and
// its corresponding diagnostic information, and prints the decision along with
// the reasons and any errors encountered during evaluation.
func outputDecision(decision cedar.Decision, diag cedar.Diagnostic) {
	fmt.Println("Decision:", decision.String())

	// If there are errors in the diagnostic, print them and return.
	if len(diag.Errors) > 0 {
		fmt.Println("Errors encountered during evaluation:")
		for _, err := range diag.Errors {
			fmt.Println(" *", err)
			fmt.Println()
		}
	}

	// If there are no reasons, then no policies applied to the request.
	if len(diag.Reasons) == 0 {
		fmt.Println("No applicable policies found.")
		fmt.Println()
		return
	}

	if decision == cedar.Allow {
		fmt.Println("Permitted by:")
	} else {
		fmt.Println("Denied by:")
	}
	// The reasons will contain the policies that contributed to the final decision.
	for _, d := range diag.Reasons {
		fmt.Println(" *", d.PolicyID)
	}
	fmt.Println()
}
