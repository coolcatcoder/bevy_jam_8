// TODO: The map will be a low resolution image that starts off fully opaque black, and we copy a
// circle around the player from the rendered map to fill this image. We could instead fill the
// circle with transparency and just render the whole map behind, but by rendering the map to the
// opaque texture, we can keep a memory of what used to be outside of the vision range without
// updating anything moving out of range.
