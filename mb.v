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
//

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

impl Rhyzome {
    fn open(db_path string) ?{
        mut conn := sqlite.connect(db_path) or {
            return err
        }

        conn.exec('CREATE TABLE IF NOT EXISTS nodes (id TEXT PRIMARY KEY, value TEXT)') or {
            return err
        }

        conn.exec('CREATE TABLE IF NOT EXISTS relations (name TEXT, from_id TEXT, to_id TEXT)') or {
            return err
        }

        return Rhyzome{conn: conn}
    }

    fn close() {
        this.conn.close()
    }

    fn set(id string, value string) ? {
        stmt := this.conn.prepare('INSERT OR REPLACE INTO nodes (id, value) VALUES (?, ?)') or {
            return err
        }

        stmt.bind(1, id) or { return err }
        stmt.bind(2, value) or { return err }
        stmt.step() or { return err }
        stmt.finalize()

        Ok()
    }

    fn get(id string) ?string {
        stmt := this.conn.prepare('SELECT value FROM nodes WHERE id = ?') or {
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

    fn delete(id string) ? {
        stmt := this.conn.prepare('DELETE FROM nodes WHERE id = ?') or {
            return err
        }

        stmt.bind(1, id) or { return err }
        stmt.step() or { return err }
        stmt.finalize()

        Ok()
    }

    fn relate(from_id string, relation_name string, to_id string) ? {
        stmt := this.conn.prepare('INSERT INTO relations (name, from_id, to_id) VALUES (?, ?, ?)') or {
            return err
        }

        stmt.bind(1, relation_name) or { return err }
        stmt.bind(2, from_id) or { return err }
        stmt.bind(3, to_id) or { return err }
        stmt.step() or { return err }
        stmt.finalize()

        Ok()
    }

    fn delete_relation(relation_name string) ? {
        stmt := this.conn.prepare('DELETE FROM relations WHERE name = ?') or {
            return err
        }

        stmt.bind(1, relation_name) or { return err }
        stmt.step() or { return err }
        stmt.finalize()

        Ok()
    }

    fn query_relation(from_id string) ?[]Relation {
        stmt := this.conn.prepare('SELECT name, to_id FROM relations WHERE from_id = ?') or {
            return err
        }

        stmt.bind(1, from_id) or { return err }

        var relations []Relation

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
    defer api.rhyzome.close()

    // Initialize HTTP server
    server := http.server{
        addr: ":8080"
    }

    // API endpoints
    server.handle('/microblogs', api.handle_microblogs)
    server.handle('/microblogs/:id', api.handle_microblog)
    server.handle('/microblogs/:id/relations', api.handle_relations)
    server.handle('/microblogs/:id/relations/:relation_name', api.handle_relation)
    server.handle('/tokens', api.handle_tokens)

    // Start the server
    server.start()
}

fn (mut api &MicroblogAPI) handle_microblogs(rw http.ResponseWriter, req http.Request) {
    if req.method == .GET {
        microblogs := api.rhyzome.query_relation("microblog")
        json_response(rw, microblogs)
    } else if req.method == .POST {
        token := req.header.get('Authorization')
        if !api.validate_token(token, "create_microblog") {
            http.response_unauthorized(rw)
            return
        }

        microblog := req.body.to_json().get_str('text')
        id := uuid()
        api.rhyzome.set(id, microblog) or { panic(err) }
        api.rhyzome.relate(id, "microblog", "")
        json_response(rw, map{"id": id})
    }
}

fn (mut api &MicroblogAPI) handle_microblog(rw http.ResponseWriter, req http.Request) {
    tweet_id := req.vars['id']
    if req.method == .GET {
        microblog := api.rhyzome.get(tweet_id)
        json_response(rw, map{"id": tweet_id, "text": microblog})
    } else if req.method == .DELETE {
        token := req.header.get('Authorization')
        if !api.validate_token(token, "delete_microblog") {
            http.response_unauthorized(rw)
            return
        }

        api.rhyzome.delete(tweet_id) or { panic(err) }
        json_response(rw, map{"message": "Microblog deleted"})
    }
}

fn (mut api &MicroblogAPI) handle_relations(rw http.ResponseWriter, req http.Request) {
    tweet_id := req.vars['id']
    if req.method == .GET {
        relations := api.rhyzome.query_relation(tweet_id)
        json_response(rw, relations)
    }
}

fn (mut api &MicroblogAPI) handle_relation(rw http.ResponseWriter, req http.Request) {
    tweet_id := req.vars['id']
    relation_name := req.vars['relation_name']

    if req.method == .POST {
        token := req.header.get('Authorization')
        if !api.validate_token(token, "create_relation") {
            http.response_unauthorized(rw)
            return
        }

        related_id := req.body.to_json().get_str('related_id')
        api.rhyzome.relate(tweet_id, relation_name, related_id)
        json_response(rw, map{"message": "Related microblog added"})
    } else if req.method == .DELETE {
        token := req.header.get('Authorization')
        if !api.validate_token(token, "delete_relation") {
            http.response_unauthorized(rw)
            return
        }

        api.rhyzome.delete_relation(relation_name)
        json_response(rw, map{"message": "Related microblogs deleted"})
    }
}

fn (mut api &MicroblogAPI) handle_tokens(rw http.ResponseWriter, req http.Request) {
    if req.method == .POST {
        password := req.body.to_json().get_str('password')
        if password != api.admin_password {
            http.response_unauthorized(rw)
            return
        }

        permissions := req.query.get_list('permissions')

        // Check if permissions are provided
        if len(permissions) == 0 {
            // Assign default permissions
            permissions = ['create_microblog', 'relate_microblog']
        }

        // Generate token with provided or default permissions
        token := Token{
            id: generate_token(),
            used: false,
            permissions: permissions,
        }
        api.tokens << token
        json_response(rw, map{"token": token.id})
    }
}

fn (api MicroblogAPI) validate_token(token string, required_permission string) bool {
    for i, t := range api.tokens {
        if t.id == token && !t.used && has_permission(t.permissions, required_permission) {
            api.tokens[i].used = true
            return true
        }
    }
    return false
}

fn has_permission(permissions []string, required_permission string) bool {
    for _, p := range permissions {
        if p == required_permission {
            return true
        }
    }
    return false
}

fn json_response(rw http.ResponseWriter, data any) {
    rw.headers['Content-Type'] = 'application/json'
    rw.write(data.to_json())
}

fn uuid() string {
    return std.uuid.new_v4().to_string()
}

fn generate_token() string {
    bytes := crypto.random.bytes(16)
    return encoding.base64.encode(bytes, encoding.base64.url_safe)
}
