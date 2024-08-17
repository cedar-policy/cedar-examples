package entitystore

import (
	"encoding/json"
	"github.com/cedar-policy/cedar-examples/tinytodo-go/internal/app/server/entitystore/entitytype"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	"testing"
)

func Test_App(t *testing.T) {
	t.Run("unmarshal App", func(t *testing.T) {
		marshalled := []byte(`
{
"euid": "Application::\"TinyTodo\""
}
`)
		var app App
		require.NoError(t, json.Unmarshal(marshalled, &app))
		assert.Equal(
			t,
			NewEntityUID(entitytype.Application, "TinyTodo"),
			app.EUID,
		)
	})
}
