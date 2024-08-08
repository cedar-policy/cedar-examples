package main

import (
	"code.byted.org/binaryauthorization/tinytodo-go/internal/app/server"
	"flag"
	"fmt"
)

var (
	DefaultLogger = server.DefaultLogger
)

// startServer starts the web server by parsing these CLI arguments:
//
//   - port: Port number (default 8080)
func startServer() error {

	port := flag.Int("port", 8080, "Listen port")
	flag.Parse()

	es, ps, err := prepareCedarPolicyEntities()
	if err != nil {
		return err
	}

	w, err := server.New(
		fmt.Sprintf(":%d", *port),
		es,
		ps,
		prepareLogger(),
	)
	if err != nil {
		return err
	}

	if err := w.Serve(); err != nil {
		return err
	}

	DefaultLogger.Info("terminating")
	return nil
}
