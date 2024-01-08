# GRAPHICAL ENGINE

## TODO LIST

- [x] Read images -> for now simple PPM as before.
- [ ] Better support for image format.
- [x] Print them, entirely (that is very fast) and partly (slower)
- [x] Transparency (even more slow, but is optionnal sprite by sprite)
- [x] Z-Order
- [x] Background tiles and sprites (mobile and immobile)
- [ ] Tileset to store the current background efficiently ? -> need to think about that, but first, I want everything else to be very solid.
- [ ] Redraw only sprites and needed background tiles
- [ ] Give possibilities for text too
- [ ] Input
- [ ] Action loop
- [ ] Superposition of the images before printing ? may not be useful.
- [ ] etc.

## REMARKS

- The Numworks display is made of big RGB pixels. Like, BIG. You can easily see, with the naked eye and from pretty far away, the artefacts on every transitions between colors.
- The display is 320*240 pixels. Prefer making images considering this size directly, as scaling is not (yet) supported and would be pretty slow if it was.
- This engine should, hopefully, be enough for a lot of projects. I do know that a lot of optimisations could be done for every specific case. I will try to make a lot of them with time, but keeping the engine good for general purpose.
- Because it is for a calculator, I try to optimise a lot but every single cpu cycle will be used.
