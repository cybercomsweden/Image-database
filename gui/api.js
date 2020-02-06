import Pbf from "pbf";
import {
    AutocompleteTags, Entity, Entities, Tag, Tags,
} from "../src/entity.proto";

async function fetchPb(cls, resource, init) {
    const rsp = await fetch(resource, init);
    const blob = await rsp.blob();
    const buf = await blob.arrayBuffer();
    return cls.read(new Pbf(buf));
}


AutocompleteTags.fetch = async function fetchAutocompleteTags() {
    return fetchPb(AutocompleteTags, "/api/tags/autocomplete");
};

Entities.fetch = async function fetchEntities() {
    return fetchPb(Entities, "/api/media");
};

Entity.fetch = async function fetchEntity(id) {
    return fetchPb(Entity, `/api/media/${id}`);
};

Tags.fetch = async function fetchTags() {
    return fetchPb(Tags, "/api/tags");
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
