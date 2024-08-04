// Package taskstate contains the enum TaskState that represents the different states supported by entitystore.Task.
package taskstate

import (
	"fmt"
	"strings"
)

// TaskState is an enum that represents the different states supported by entitystore.Task.
type TaskState int

const (
	Unknown TaskState = iota
	Checked
	Unchecked
)

var (
	Name = map[TaskState]string{
		Unknown:   "unknown",
		Checked:   "Checked",
		Unchecked: "Unchecked", // expected by client
	}
)

func (t TaskState) String() string {
	return Name[t]
}

// Parse parses a given string as TaskState using strings.EqualFold() (case insensitive)
func Parse(et string) TaskState {
	for res := range Name {
		if strings.EqualFold(res.String(), et) {
			return res
		}
	}
	return Unknown
}

// MarshalJSON converts a TaskState into a textual representation.
//
// Based on https://pkg.go.dev/encoding/json.
//
// Note that the value receiver is not an error -- MarshalJSON is supposed to act on values.
func (t TaskState) MarshalJSON() ([]byte, error) {
	return []byte(fmt.Sprintf("%q", t.String())), nil
}
