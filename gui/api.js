import Pbf from "pbf";
import queryString from "query-string";
import {
    AutocompleteTags, Entity, Entities, Tag, Tags,
} from "../src/entity.proto";

async function fetchPb(cls, resource, init) {
    const rsp = await fetch(resource, init);
    const blob = await rsp.blob();
    const buf = await blob.arrayBuffer();
    return cls.read(new Pbf(buf));
}

async function putPb(cls, resource, obj) {
    const pbf = new Pbf();
    cls.write(obj, pbf);

    const rsp = await fetch(resource, {
        method: "PUT",
        headers: {
            "Content-Type": "application/protobuf",
        },
        body: pbf.finish(),
    });

    const blob = await rsp.blob();
    const buf = await blob.arrayBuffer();
    return cls.read(new Pbf(buf));
}


AutocompleteTags.fetch = async function fetchAutocompleteTags() {
    return fetchPb(AutocompleteTags, "/api/tags/autocomplete");
};

Entities.fetch = async function fetchEntities(query) {
    let queryParams = "";
    if (query) {
        queryParams = `?${queryString.stringify({ q: query })}`;
    }
    return fetchPb(Entities, `/api/media${queryParams}`);
};

Entity.fetch = async function fetchEntity(id) {
    return fetchPb(Entity, `/api/media/${id}`);
};

Entity.save = async function saveEntity(entity) {
    return putPb(Entity, `/api/media/${entity.id}`, entity);
};

Tags.fetch = async function fetchTags() {
    return fetchPb(Tags, "/api/tags");
};

Tag.add = async function addTag(parent, tag) {
    return fetchPb(Tag, `/api/tags/${parent}/${tag}`, { method: "POST" });
};

Tag.fetch = async function fetchTag(name) {
    return fetchPb(Tag, `/api/tags/${name}`);
};


export {
    AutocompleteTags,
    Entity,
    Entities,
    Tag,
    Tags,
};
