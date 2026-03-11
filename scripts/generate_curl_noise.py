import numpy as np
from PIL import Image

# 生成基础的高频白噪声并进行平滑，作为简单的 Curl Noise 演示
size = 256
noise = np.random.randint(0, 256, (size, size, 3), dtype=np.uint8)
img = Image.fromarray(noise)
img.save('assets/textures/vfx/noise/curl_noise.png')
