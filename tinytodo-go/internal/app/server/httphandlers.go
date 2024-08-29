package server

import (
	"context"
	"encoding/json"
	"fmt"
	"net/http"
)

// GenericResponse represents a JSON response to any API endpoint.
type GenericResponse struct {
	Error   string `json:"error,omitempty"`
	Message string `json:"message,omitempty"`
}

// writeGenericResponse marshals a GenericResponse into a http.ResponseWriter.
//
// Calls ServerError if the marshal process fails.
func (s *Server) writeGenericResponse(
	_ context.Context,
	w http.ResponseWriter,
	resp *GenericResponse,
	statusCode int,
) {
	out, err := json.MarshalIndent(
		resp,
		"",
		"    ",
	)
	if err != nil {
		s.ServerError(w, fmt.Errorf("failed to marshal response JSON"))
		return
	}
	w.Header().Set("Content-Type", "application/json")
	w.WriteHeader(statusCode)
	if _, err := w.Write(out); err != nil {
		s.ServerError(w, fmt.Errorf("failed to write to response: %w", err))
		return
	}
}
