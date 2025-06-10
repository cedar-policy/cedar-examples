package user

import (
	"encoding/json"
	"github.com/cedar-policy/cedar-examples/tinytodo-go/internal/app/server/entitystore/entity"
	"github.com/cedar-policy/cedar-examples/tinytodo-go/internal/app/server/entitystore/entitytype"
	"github.com/cedar-policy/cedar-examples/tinytodo-go/internal/app/server/entitystore/entityuid"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	"testing"
)

func TestUser(t *testing.T) {
	t.Run("check interface", func(t *testing.T) {
		var e entity.Entity
		u := New(
			UserUID{
				EntityUID: entityuid.New(entitytype.User, "andrew"),
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
				entityuid.New(entitytype.User, "kesha"),
			},
			u.EUID,
		)
		assert.Equal(t, "ABC17", u.Location)
		assert.Equal(t, 5, u.JobLevel)
		assert.Contains(
			t,
			u.Parents,
			entityuid.New(entitytype.Application, "TinyTodo"),
		)
		assert.Contains(
			t,
			u.Parents,
			entityuid.New(entitytype.Team, "temp"),
		)
	})
}
