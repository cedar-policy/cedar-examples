// Package role contains the enum Role that represents the different roles supported by entitystore.EntityStore.
package role

import (
	"strings"
)

// Role is an enum that represents the different roles supported by entitystore.EntityStore.
type Role int

const (
	Unknown Role = iota
	Reader
	Editor
)

var (
	Name = map[Role]string{
		Unknown: "unknown",
		Reader:  "Reader",
		Editor:  "Editor",
	}
)

func (r Role) String() string {
	return Name[r]
}

// Parse parses a given string as Role using strings.EqualFold() (case insensitive)
func Parse(r string) Role {
	for res := range Name {
		if strings.EqualFold(res.String(), r) {
			return res
		}
	}
	return Unknown
}
