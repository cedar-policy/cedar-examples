package harness

import "context"
import "log"
import "time"
import "fmt"

import "github.com/open-policy-agent/opa/rego"

type Module = func(r *rego.Rego)


func IsAuthorized(input map[string]interface{}, query rego.PreparedEvalQuery) TestData  {
    ctx := context.Background()

    start := time.Now()
    evalled := rego.EvalInput(input)
    results, err := query.Eval(ctx, evalled)
    duration := time.Since(start)

    if err != nil {
        // Handle evaluation error.
        log.Print("Error evaluating policies")
        panic(err)
    } 

    var authResult bool

    if len(results) == 0 {
        authResult = false
    } else if was_allowed, ok := results[0].Bindings["x"].(bool); !ok {
        // Handle unexpected result type.
        panic("Unexpected type for `allow`\n")
    } else if was_allowed {
        authResult = true
    } else { 
        authResult = false
    }

    return TestData { Decision : authResult, Dur : duration }
}

func ReadPolicy(src string, namespace string) rego.PreparedEvalQuery { 
    ctx := context.Background()

    filename := fmt.Sprintf("%s.rego", namespace)
    query_string := fmt.Sprintf("x = data.%s.allow", namespace)

    module := rego.Module(filename, src)
    query, err := rego.New(
        rego.Query(query_string),
        module,
    ).PrepareForEval(ctx)
    if err != nil { 
        log.Print("Error constructing query")
        panic(err)
    }
    return query
}
