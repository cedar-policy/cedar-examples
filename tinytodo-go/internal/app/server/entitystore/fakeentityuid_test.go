package entitystore

import (
	"code.byted.org/binaryauthorization/tinytodo-go/internal/app/server/entitystore/entitytype"
	"encoding/json"
	"github.com/cedar-policy/cedar-go"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	"testing"
)

type MyEntity struct {
	cedar.EntityUID
}

func TestMyEntity(t *testing.T) {

	m := MyEntity{
		EntityUID: cedar.NewEntityUID(entitytype.User.String(), "kesha"),
	}

	res1, err := m.MarshalJSON()
	require.NoError(t, err)

	t.Log(string(res1)) // {"type":"User","id":"kesha"}

	e := EntityUID{
		EntityUID: cedar.NewEntityUID(entitytype.User.String(), "kesha"),
	}

	res2, err := e.MarshalJSON()
	require.NoError(t, err)

	t.Log(string(res2)) // "User::\"kesha\""

	// will fail

	//	var m2 MyEntity
	b2 := []byte(`
"User::\"kesha\""
`)
	//
	//	assert.NoError(t, json.Unmarshal(b2, &m2))
	//	t.Log(m2)

	var m3 MyEntity
	b3 := []byte(`
{"type":"User","id":"kesha"}
`)

	assert.NoError(t, json.Unmarshal(b3, &m3))
	t.Log(m3)

	var e2 EntityUID
	assert.NoError(t, json.Unmarshal(b2, &e2))
	t.Log(e2)

}
