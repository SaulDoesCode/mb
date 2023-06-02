// MIT License
//
// Copyright (c) 2023 Saul van der Walt
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// This microblogging server allows users to create, retrieve, update, and delete
// microblogs in a distributed and decentralized manner. It provides an API for
// managing microblogs and their relations, with a token-based authentication system
// for ensuring secure access to the server's functionalities.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

import sqlite
import http
import crypto.random
import encoding.base64

struct Rhyzome {
	conn sqlite.Connection
}

struct Relation {
	name string
	from_id string
	to_id string
}

fn (rhyzome Rhyzome) open(db_path string) ? {
	mut conn := sqlite.connect(db_path) or {
		return err
	}

	conn.exec('CREATE TABLE IF NOT EXISTS nodes (id TEXT PRIMARY KEY, value TEXT)') or {
		return err
	}

	conn.exec('CREATE TABLE IF NOT EXISTS relations (name TEXT, from_id TEXT, to_id TEXT)') or {
		return err
	}

	rhyzome.conn = conn
	return
}

fn (rhyzome Rhyzome) close() {
	rhyzome.conn.close()
}

fn (rhyzome Rhyzome) set(id string, value string) ? {
	stmt := rhyzome.conn.prepare('INSERT OR REPLACE INTO nodes (id, value) VALUES (?, ?)') or {
		return err
	}

	stmt.bind(1, id) or { return err }
	stmt.bind(2, value) or { return err }
	stmt.step() or { return err }
	stmt.finalize()

	return
}

fn (rhyzome Rhyzome) get(id string) ?string {
	stmt := rhyzome.conn.prepare('SELECT value FROM nodes WHERE id = ?') or {
		return err
	}

	stmt.bind(1, id) or { return err }
	if stmt.step() ? {
		value := stmt.get_str(0)
		stmt.finalize()
		return value
	}

	stmt.finalize()
	return null
}

fn (rhyzome Rhyzome) delete(id string) ? {
	stmt := rhyzome.conn.prepare('DELETE FROM nodes WHERE id = ?') or {
		return err
	}

	stmt.bind(1, id) or { return err }
	stmt.step() or { return err }
	stmt.finalize()

	return
}

fn (rhyzome Rhyzome) relate(from_id string, relation_name string, to_id string) ? {
	stmt := rhyzome.conn.prepare('INSERT INTO relations (name, from_id, to_id) VALUES (?, ?, ?)') or {
		return err
	}

	stmt.bind(1, relation_name) or { return err }
	stmt.bind(2, from_id) or { return err }
	stmt.bind(3, to_id) or { return err }
	stmt.step() or { return err }
	stmt.finalize()

	return
}

fn (rhyzome Rhyzome) delete_relation(relation_name string) ? {
	stmt := rhyzome.conn.prepare('DELETE FROM relations WHERE name = ?') or {
		return err
	}

	stmt.bind(1, relation_name) or { return err }
	stmt.step() or { return err }
	stmt.finalize()

	return
}

fn (rhyzome Rhyzome) query_relation(from_id string) ?[]Relation {
	stmt := rhyzome.conn.prepare('SELECT name, to_id FROM relations WHERE from_id = ?') or {
		return err
	}

	stmt.bind(1, from_id) or { return err }

	mut relations := [] 

	for stmt.next() {
		relation_name := stmt.get_str(0)
		to_id := stmt.get_str(1)

		relation := Relation{
			name: relation_name,
			from_id: from_id,
			to_id: to_id,
		}

		relations << relation
	}

	stmt.finalize()
	return relations
}

fn (rhyzome Rhyzome) iter_values() ?[]string {
	stmt := rhyzome.conn.prepare('SELECT value FROM nodes') or {
		return err
	}

	mut values := []

	for stmt.next() {
		value := stmt.get_str(0)
		values << value
	}

	stmt.finalize()
	return values
}

struct Microblog {
    id string
    text string
}

struct Token {
    id string
    used bool
    permissions []string
}

struct MicroblogAPI {
    rhyzome Rhyzome
    tokens []Token
    admin_password string
}

fn main() {
    mut api := MicroblogAPI{
        rhyzome: Rhyzome{},
        tokens: [],
        admin_password: "your_admin_password_here",
    }
    api.rhyzome.open(":memory:") or { panic(err) }
    defer {
	 api.rhyzome.close()
	}

    // Initialize HTTP server
    server := http.server{
        addr: ":8080"
    }

    // API endpoints
    server.handle('/microblogs', api.handle_microblogs)
    server.handle('/microblogs/:id', api.handle_microblog)
    server.handle('/microblogs/:id/relations', api.handle_relations)
    server.handle('/microblogs/:id/relations/:relation_name', api.handle_relation)
	server.handle('/microblogs/all', api.handle_all_microblogs)
    server.handle('/tokens', api.handle_tokens)

    // Start the server
    server.start()
}

fn (mut api MicroblogAPI) handle_microblogs(rw http.ResponseWriter, req http.Request) {
    if req.method == .GET {
        microblogs := api.rhyzome.query_relation("microblog")
        json_response(rw, microblogs)
    } else if req.method == .POST {
        token := req.header.get('Authorization')
        if !api.validate_token(token) {
            http.response_unauthorized(rw)
            return
        }

        microblog := req.body.to_json().get_str('text')
        id := uuid()
        api.rhyzome.set(id, microblog) or { panic(err) }
        api.rhyzome.relate(id, "microblog", "")
        json_response(rw, {'id': id})
    }
}

fn (mut api MicroblogAPI) handle_all_microblogs(rw http.ResponseWriter, req http.Request) {
	if req.method == .GET {
		microblogs, err := api.rhyzome.iter_values()
		if err != null {
			http.response_internal_server_error(rw)
			return
		}

		json_response(rw, microblogs)
	}
}

fn (mut api MicroblogAPI) handle_microblog(rw http.ResponseWriter, req http.Request) {
    blog_id := req.vars['id']
    if req.method == .GET {
        microblog := api.rhyzome.get(blog_id)
        json_response(rw, {'id': blog_id, 'text': microblog})
    } else if req.method == .DELETE {
        token := req.header.get('Authorization')
        if !api.validate_token(token) {
            http.response_unauthorized(rw)
            return
        }

        api.rhyzome.delete(blog_id) or { panic(err) }
        json_response(rw, {'message': "Microblog deleted"})
    }
}

fn (mut api MicroblogAPI) handle_relations(rw http.ResponseWriter, req http.Request) {
    blog_id := req.vars['id']
    if req.method == .GET {
        relations := api.rhyzome.query_relation(blog_id)
        json_response(rw, relations)
    }
}

fn (mut api MicroblogAPI) handle_relation(rw http.ResponseWriter, req http.Request) {
    blog_id := req.vars['id']
    relation_name := req.vars['relation_name']

    if req.method == .POST {
        token := req.header.get('Authorization')
        if !api.validate_token(token) {
            http.response_unauthorized(rw)
            return
        }

        related_id := req.body.to_json().get_str('related_id')
        api.rhyzome.relate(blog_id, relation_name, related_id)
        json_response(rw, {'message': "Related microblog added"})
    } else if req.method == .DELETE {
        token := req.header.get('Authorization')
        if !api.validate_token(token) {
            http.response_unauthorized(rw)
            return
        }

        api.rhyzome.delete_relation(relation_name)
        json_response(rw, {'message': "Related microblog deleted"})
    }
}

fn (mut api MicroblogAPI) handle_tokens(rw http.ResponseWriter, req http.Request) {
    if req.method == .POST {
        password := req.body.to_json().get_str('password')
        if password == api.admin_password {
            permissions := req.body.to_json().get_str('permissions') or {
				"create_microblog,relate_microblog"
			}
            token := api.create_token(permissions)
            json_response(rw, {'token': token})
        } else {
            http.response_unauthorized(rw)
        }
    }
}

fn (api MicroblogAPI) validate_token(token string) bool {
    for t in api.tokens {
        if t.id == token && !t.used {
            return true
        }
    }
    return false
}

fn (mut api MicroblogAPI) create_token(permissions string) string {
    id := uuid()
    token := Token{
        id: id,
        used: false,
        permissions: permissions.split(','),
    }
    api.tokens << token
    return id
}

fn json_response(rw http.ResponseWriter, data interface{}) {
    json_str := encoding.base64.encode(data.to_json().to_string())
    rw.header.set('Content-Type', 'application/json')
    rw.write(json_str)
}

fn uuid() string {
    mut buf := [16]byte{}
    crypto.random.fill(buf)
    return encoding.base64.encode(buf)
}
