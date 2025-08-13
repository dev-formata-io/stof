# Image Library (Image)
Functions for working with images, loaded into Stof via the custom Data<Image> type. This can be done with several image formats. Requires the "image" feature flag to be enabled.

# Image.blob(img: Data\<Image>) -> blob
Transform this image into a binary blob value (raw binary is a PNG).


# Image.blur(img: Data\<Image>, blur: float) -> void
Blur this image with the given blur value (sigma in a gaussian blur).


# Image.bmp(img: Data\<Image>) -> blob
Transform this image into a binary blob value that is in BMP format.


# Image.brighten(img: Data\<Image>, brighten: int) -> void
Brighten this image with the given value (positive to increase each pixel and negative to decrease).


# Image.contrast(img: Data\<Image>, contrast: float) -> void
Set the contrast of this image (positive to increase, negative to decrease).


# Image.fast_blur(img: Data\<Image>, blur: float) -> void
Blur this image with the given blur value (sigma in a gaussian blur).


# Image.flip_horizontal(img: Data\<Image>) -> void
Flip this image horizontally.


# Image.flip_vertical(img: Data\<Image>) -> void
Flip this image vertically.


# Image.from_blob(bytes: blob) -> Data\<Image>
Create an image (on the calling/current object) given a binary blob, attempting to auto-detect the image's format.


# Image.gif(img: Data\<Image>) -> blob
Transform this image into a binary blob value that is in GIF format.


# Image.grayscale(img: Data\<Image>) -> void
Turn this image into grayscale.


# Image.height(img: Data\<Image>) -> int
Height in pixels of this image.


# Image.ico(img: Data\<Image>) -> blob
Transform this image into a binary blob value that is in ICO format.


# Image.invert(img: Data\<Image>) -> void
Invert this image.


# Image.jpeg(img: Data\<Image>) -> blob
Transform this image into a binary blob value that is in JPEG format.


# Image.png(img: Data\<Image>) -> blob
Transform this image into a binary blob value that is in PNG format.


# Image.resize(img: Data\<Image>, width: int, height: int) -> bool
Resize this image, preserving it's aspect ratio. Will return true if the image was successfully resized.


# Image.resize_exact(img: Data\<Image>, width: int, height: int) -> bool
Resize this image, without preserving it's aspect ratio. Will return true if the image was successfully resized.


# Image.rotate_180(img: Data\<Image>) -> void
Rotate this image 180 degrees clockwise.


# Image.rotate_270(img: Data\<Image>) -> void
Rotate this image 270 degrees clockwise.


# Image.rotate_90(img: Data\<Image>) -> void
Rotate this image 90 degrees clockwise.


# Image.thumbnail(img: Data\<Image>, width: int, height: int) -> bool
Resize this image into a thumbnail, preserving it's aspect ratio. Will return true if the image was successfully resized.


# Image.thumbnail_exact(img: Data\<Image>, width: int, height: int) -> bool
Resize this image into a thumbnail, without preserving it's aspect ratio. Will return true if the image was successfully resized.


# Image.tiff(img: Data\<Image>) -> blob
Transform this image into a binary blob value that is in TIFF format.


# Image.webp(img: Data\<Image>) -> blob
Transform this image into a binary blob value that is in WEBP format.


# Image.width(img: Data\<Image>) -> int
Width in pixels of this image.


