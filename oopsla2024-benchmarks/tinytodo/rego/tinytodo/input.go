package tinytodo

import "time"
import "encoding/json"
import "os"
import "io"
import "fmt"

type Input struct {
    Policy string
    Namespace string
    Requests []interface{}
}

type TestData struct {
    Decision bool
    Dur time.Duration
}

func WriteOutput(data []TestData) {
    buf, err := json.Marshal(data)
    if err != nil {
        panic(err)
    }
    _, err = os.Stdout.Write(buf[:])
    if err != nil {
        panic(err)
    }
    fmt.Printf("\n")
}


func ReadInput() Input {
    data, err := io.ReadAll(os.Stdin)
    if err != nil { 
        panic(err)
    }
    var i Input
    err = json.Unmarshal(data, &i)
    if err != nil { 
        panic(err)
    }
    return i
}
