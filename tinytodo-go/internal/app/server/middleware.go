package server

import (
	"log/slog"
	"net/http"
)

// logRequest logs each HTTP request
func (s *Server) logRequest(next http.Handler) http.Handler {
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		s.logger.InfoContext(
			r.Context(),
			"hit page",
			slog.String("method", r.Method),
			slog.String("url", r.URL.String()),
		)
		next.ServeHTTP(w, r)
	})
}
