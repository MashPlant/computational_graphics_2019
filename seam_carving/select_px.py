import cv2
import sys

beg = None

def mouse_callback(event, x, y, flags, params):
  global beg
  if event == cv2.EVENT_LBUTTONDOWN:
    beg = (x, y)
  elif event == cv2.EVENT_LBUTTONUP:
    end = (x, y)
    print(min(beg[0], end[0]), min(beg[1], end[1]), max(beg[0], end[0]), max(beg[1], end[1]))
    cv2.destroyAllWindows()
    sys.exit(0)


img = cv2.imread(sys.argv[1])
scale_width = 640 / img.shape[1]
scale_height = 480 / img.shape[0]
scale = min(scale_width, scale_height)
window_width = int(img.shape[1] * scale)
window_height = int(img.shape[0] * scale)
cv2.namedWindow('image', cv2.WINDOW_NORMAL)
cv2.resizeWindow('image', window_width, window_height)
cv2.setMouseCallback('image', mouse_callback)
cv2.imshow('image', img)
cv2.waitKey(0)