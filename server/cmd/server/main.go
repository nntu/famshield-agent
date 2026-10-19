package main

import (
	"log"
	"net/http"

	"famshield-server/pkg/api"
	"famshield-server/pkg/store"
)

func main() {
	st := store.NewMemoryStore()
	srv := api.NewServer(st)

	addr := ":8080"
	log.Printf("Starting FamShield Control Plane server on %s ...", addr)
	if err := http.ListenAndServe(addr, srv.Handler()); err != nil {
		log.Fatalf("Server failed: %v", err)
	}
}
