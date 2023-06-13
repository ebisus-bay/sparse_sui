-- Your SQL goes here

CREATE TABLE display_objects (
    object_id address PRIMARY KEY,
    object_type VARCHAR NOT NULL,
    owner_address address NULL,
    name VARCHAR NOT NULL,
    link VARCHAR NOT NULL,
    image_url VARCHAR NOT NULL,
    description VARCHAR NOT NULL,
    project_url VARCHAR NOT NULL,
    creator VARCHAR NOT NULL,

    -- Foreign key
    FOREIGN KEY (object_type) references collections(base_type)
);