package entitystore

import (
	"code.byted.org/binaryauthorization/tinytodo-go/internal/app/server/entitystore/entitytype"
	"encoding/json"
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
