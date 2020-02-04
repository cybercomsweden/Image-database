import Pbf from 'pbf';
import {Entity, Entities} from '../src/entity.proto';

async function _fetch(cls, resource, init) {
    const rsp = await fetch(resource, init);
    const blob = await rsp.blob();
    const buf = await blob.arrayBuffer();
    return cls.read(new Pbf(buf));
}


Entities.fetch = async function() {
    return await _fetch(Entities, "/api/media");
}

Entity.fetch = async function(id) {
    return await _fetch(Entity, `/api/media/${id}`);
}


export {
    Entity,
    Entities,
};
