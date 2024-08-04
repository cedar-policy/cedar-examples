package server

import (
	"log/slog"
	"net/http"
	"runtime/debug"
)

// ClientError logs error based on its status code and returns the status code in the response.
func (s *Server) ClientError(w http.ResponseWriter, status int, err error) {
	s.logger.Info("client error", slog.String("status", http.StatusText(status)), slog.Any("error", err))
	http.Error(w, http.StatusText(status), status)
	return
}

// ServerError logs the error and a stack trace, and returns a StatusInternalServerError in the response.
func (s *Server) ServerError(w http.ResponseWriter, err error) {
	s.logger.Error("server error", slog.Any("error", err), slog.String("stack", string(debug.Stack())))
	http.Error(w, http.StatusText(http.StatusInternalServerError), http.StatusInternalServerError)
	return
}
