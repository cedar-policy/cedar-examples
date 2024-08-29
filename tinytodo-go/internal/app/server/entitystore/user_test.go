package entitystore

import (
	"encoding/json"
	"github.com/cedar-policy/cedar-examples/tinytodo-go/internal/app/server/entitystore/entitytype"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	"testing"
)

func TestUser(t *testing.T) {
	t.Run("check interface", func(t *testing.T) {
		var e Entity
		u := NewUser(
			UserUID{
				EntityUID: NewEntityUID(entitytype.User, "andrew"),
			},
			"test_location",
			0,
			nil,
		)
		e = u
		assert.NotNil(t, e)
	})
}

func TestUser_Unmarshal(t *testing.T) {
	t.Run("unmarshal User", func(t *testing.T) {
		marshalled := []byte(`
{
  "euid": "User::\"kesha\"",
  "location": "ABC17",
  "joblevel": 5,
  "parents": [
	"Application::\"TinyTodo\"",
	"Team::\"temp\""
  ]
}
`)
		var u User
		require.NoError(t, json.Unmarshal(marshalled, &u))
		assert.Equal(
			t,
			UserUID{
				NewEntityUID(entitytype.User, "kesha"),
			},
			u.EUID,
		)
		assert.Equal(t, "ABC17", u.Location)
		assert.Equal(t, 5, u.JobLevel)
		assert.Contains(
			t,
			u.Parents,
			NewEntityUID(entitytype.Application, "TinyTodo"),
		)
		assert.Contains(
			t,
			u.Parents,
			NewEntityUID(entitytype.Team, "temp"),
		)
	})
}
