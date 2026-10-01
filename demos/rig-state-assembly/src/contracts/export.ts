import { RIG_STATE_MANIFEST, validateRigStateManifest, type RigStateManifest } from "./rig-state.ts";

export interface StateCatalogueEntry {
  id: string;
  label: string;
  activityId: string;
  dysfunctionIds: readonly string[];
}

export interface PortableCompositionEntry {
  id: string;
  activityId: string;
  layerIds: readonly string[];
  dysfunctionIds: readonly string[];
}

export interface RigStateArtifacts {
  catalogue: readonly StateCatalogueEntry[];
  compositions: readonly PortableCompositionEntry[];
}

export function createRigStateArtifacts(manifest: RigStateManifest = RIG_STATE_MANIFEST): RigStateArtifacts {
  const errors = validateRigStateManifest(manifest);
  if (errors.length) throw new Error(`Invalid rig-state manifest: ${errors.join("; ")}`);
  const activities = manifest.activities ?? [];
  const dysfunctions = manifest.dysfunctions ?? [];
  const catalogue = manifest.states.map((state) => ({
    id: state.id,
    label: state.label,
    activityId: state.activityId,
    dysfunctionIds: state.dysfunctionIds ?? [],
  }));
  const compositions = manifest.states.map((state) => {
    const activity = activities.find((candidate) => candidate.id === state.activityId);
    const attachedDysfunctions = (state.dysfunctionIds ?? [])
      .map((id) => dysfunctions.find((candidate) => candidate.id === id))
      .filter((candidate): candidate is NonNullable<typeof candidate> => Boolean(candidate));
    return {
      id: state.id,
      activityId: state.activityId,
      layerIds: [...(activity?.layerIds ?? []), ...attachedDysfunctions.flatMap((item) => item.layerIds)],
      dysfunctionIds: state.dysfunctionIds ?? [],
    };
  });

  return { catalogue, compositions };
}

export function validateRigStateArtifacts(artifacts: RigStateArtifacts, manifest: RigStateManifest = RIG_STATE_MANIFEST): string[] {
  const manifestErrors = validateRigStateManifest(manifest);
  if (manifestErrors.length) return manifestErrors.map(error => `Invalid rig-state manifest: ${error}`);
  const errors: string[] = [];
  const duplicateIds = (entries: readonly { id: string }[], kind: string): void => {
    const seen = new Set<string>();
    entries.forEach((entry) => {
      if (seen.has(entry.id)) errors.push(`duplicate ${kind} id: ${entry.id}`);
      seen.add(entry.id);
    });
  };
  duplicateIds(artifacts.catalogue, "catalogue");
  duplicateIds(artifacts.compositions, "composition");

  const catalogueIds = new Set(artifacts.catalogue.map((entry) => entry.id));
  const compositionIds = new Set(artifacts.compositions.map((entry) => entry.id));
  catalogueIds.forEach((id) => {
    if (!compositionIds.has(id)) errors.push(`catalogue id missing composition: ${id}`);
  });
  compositionIds.forEach((id) => {
    if (!catalogueIds.has(id)) errors.push(`composition id missing catalogue: ${id}`);
  });

  const validLayerIds = new Set((manifest.layers ?? []).map((l) => l.id));

  artifacts.catalogue.forEach(entry => {
    if (new Set(entry.dysfunctionIds).size !== entry.dysfunctionIds.length) errors.push(`catalogue ${entry.id} has duplicate dysfunction references`);
  });
  artifacts.compositions.forEach((composition) => {
    if (new Set(composition.layerIds).size !== composition.layerIds.length) errors.push(`composition ${composition.id} has duplicate layer references`);
    if (new Set(composition.dysfunctionIds).size !== composition.dysfunctionIds.length) errors.push(`composition ${composition.id} has duplicate dysfunction references`);
    const catalogue = artifacts.catalogue.find((entry) => entry.id === composition.id);
    if (catalogue && catalogue.activityId !== composition.activityId) {
      errors.push(`composition activity differs from catalogue for ${composition.id}`);
    }
    if (!composition.layerIds.length) errors.push(`composition has no layers: ${composition.id}`);
    if (validLayerIds.size > 0) {
      for (const layerId of composition.layerIds) {
        if (!validLayerIds.has(layerId)) {
          errors.push(`composition ${composition.id} references invalid layer: ${layerId}`);
        }
      }
    }
  });

  return errors;
}
