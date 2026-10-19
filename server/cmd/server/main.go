package main

import (
	"log"
	"net/http"

	"childprotect-server/pkg/api"
	"childprotect-server/pkg/store"
)

func main() {
	st := store.NewMemoryStore()
	srv := api.NewServer(st)

	addr := ":8080"
	log.Printf("Starting ChildProtect Control Plane server on %s ...", addr)
	if err := http.ListenAndServe(addr, srv.Handler()); err != nil {
		log.Fatalf("Server failed: %v", err)
	}
}
