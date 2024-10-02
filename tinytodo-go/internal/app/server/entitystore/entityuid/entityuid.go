package entityuid

import (
	"encoding/json"
	"fmt"
	"github.com/cedar-policy/cedar-examples/tinytodo-go/internal/app/server/entitystore/entitytype"
	"github.com/cedar-policy/cedar-go/types"
	"strings"
)

// EntityUID is a transparent wrapper around types.EntityUID used to represent entities in EntityStore, so that we
// can define our own JSON marshaller.
//
// See EntityUID.MarshalJSON to understand why we need to define our own marshaller.
type EntityUID struct {
	types.EntityUID
}

// New creates an EntityUID from an entitytype.EntityType and ID.
//
// It is a simple wrapper around types.NewEntityUID with constraints on the valid types.
func New(typ entitytype.EntityType, id string) EntityUID {
	return EntityUID{
		EntityUID: types.NewEntityUID(
			types.EntityType(typ.String()),
			types.String(id),
		),
	}
}

// ParseEntityUID converts the Cedar language representation of a types.EntityType into an EntityUID.
// Additionally, it checks that the type of the EntityUID matches one of the enums defined in entitytype.EntityType.
//
// Example Cedar language representation of a types.EntityUID:
//
//	"User::\"kesha\""
func ParseEntityUID(uid string) (EntityUID, error) {
	parts := strings.Split(uid, "::")
	if len(parts) != 2 {
		return EntityUID{}, fmt.Errorf("wrong number of components, expected %d, got %d", 2, len(parts))
	}

	entityType := entitytype.Parse(parts[0])
	if entityType == entitytype.Unknown {
		return EntityUID{}, fmt.Errorf("invalid entity type: %s", parts[0])
	}

	id := parts[1]
	if len(id) > 0 && id[0] == '"' {
		id = id[1:]
	}
	if len(id) > 0 && id[len(id)-1] == '"' {
		id = id[:len(id)-1]
	}
	return EntityUID{EntityUID: types.NewEntityUID(
		types.EntityType(entityType.String()),
		types.String(id),
	)}, nil
}

// UnmarshalJSON converts a Cedar language representation of a types.EntityUID into an EntityUID.
//
// Example input (enclosing double quotes are optional):
//
//	"User::\"kesha\""
//
// We cannot rely on types.EntityUID.UnmarshalJSON because the entities.json provided in the tinytodo example
// does not conform to the [Cedar language entities and context syntax].
//
// Also see [this Github issue].
//
// [Cedar language entities and context syntax]: https://docs.cedarpolicy.com/auth/entities-syntax.html
// [this Github issue]: https://github.com/cedar-policy/cedar-examples/issues/186
func (e *EntityUID) UnmarshalJSON(data []byte) error {

	var v interface{}
	if err := json.Unmarshal(data, &v); err != nil {
		return err
	}

	dataParsed, ok := v.(string)
	if !ok {
		return fmt.Errorf("entityUID must be a string")
	}

	parts := strings.Split(dataParsed, "::")
	if len(parts) != 2 {
		return fmt.Errorf("wrong number of components, expected %d, got %d", 2, len(parts))
	}

	entityType := entitytype.Parse(parts[0])
	if entityType == entitytype.Unknown {
		return fmt.Errorf("invalid entity type: %s", parts[0])
	}

	id := parts[1]
	if len(id) > 0 && id[0] == '"' {
		id = id[1:]
	}
	if len(id) > 0 && id[len(id)-1] == '"' {
		id = id[:len(id)-1]
	}

	e.Type = types.EntityType(entityType.String())
	e.ID = types.String(id)

	return nil
}

// MarshalJSON converts a EntityUID into a textual representation.
//
// Example output (enclosing double quotes included):
//
//	"User::\"kesha\""
func (e EntityUID) MarshalJSON() ([]byte, error) {
	return []byte(fmt.Sprintf("%q", e.EntityUID.String())), nil
}
