import Pbf from "pbf";
import { Entity, Entities } from "../src/entity.proto";

async function fetchPb(cls, resource, init) {
    const rsp = await fetch(resource, init);
    const blob = await rsp.blob();
    const buf = await blob.arrayBuffer();
    return cls.read(new Pbf(buf));
}


Entities.fetch = async function fetchEntities() {
    return fetchPb(Entities, "/api/media");
};

Entity.fetch = async function fetchEntity(id) {
    return fetchPb(Entity, `/api/media/${id}`);
};


export {
    Entity,
    Entities,
};
