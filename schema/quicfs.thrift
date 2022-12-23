namespace rs schema

typedef binary NodeHandle

enum NodeType {
    FILE = 1,
    DIRECTORY = 2,
}

struct NodeAttributes {
    1: required NodeHandle parentHandle,
    2: required NodeHandle nodeHandle,
    3: required NodeType nodeType,
    4: string name,
    5: i64 size,
}

exception NotFound {}

service Quicfs {
    void ping(),

    NodeHandle mount(1: list<string> path) throws (1: NotFound notFound),

    void readdir(1: NodeHandle nodeHandle) throws (1: NotFound notFound),
}

service QuicfsSubscriber {
    oneway void acceptAttributes(1: NodeAttributes attributes),
}
