CREATE PROPERTY GRAPH app.graph_shop
    VERTEX TABLES (
        app.orders KEY (id) LABEL purchase PROPERTIES (id, placed_at, total, user_id),
        app.users KEY (id) LABEL customer PROPERTIES (active, country, created_at, email, id)
    )
    EDGE TABLES (
        app.orders AS made KEY (id) SOURCE KEY (user_id) REFERENCES users (id) DESTINATION KEY (id) REFERENCES orders (id) LABEL placed PROPERTIES (id, placed_at, total, user_id)
    );
