package server

import (
	"encoding/json"
	"github.com/stretchr/testify/require"
	"testing"
)

func Test_apiTaskUpdateRequest(t *testing.T) {
	t.Run("test unmarshal", func(t *testing.T) {
		marshalled := []byte(`
{
  "uid": "User::\"andrew\"",
  "list": "List::\"0\"",
  "task": 0,
  "state": "Checked"
}
`)
		var req apiTaskUpdateRequest
		require.NoError(t, json.Unmarshal(marshalled, &req))
	})
}
