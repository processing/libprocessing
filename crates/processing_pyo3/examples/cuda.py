from mewnala import *
import torch
import torch.nn.functional as F
import math

W, H = 512, 512
KERNEL_SIZE = 15
sigma = 4.0

ax = torch.arange(-KERNEL_SIZE // 2 + 1.0, KERNEL_SIZE // 2 + 1.0, device="cuda")
xx, yy = torch.meshgrid(ax, ax, indexing="ij")
kernel = torch.exp(-(xx**2 + yy**2) / (2.0 * sigma**2))
kernel = kernel / kernel.sum()
BLUR = kernel.unsqueeze(0).unsqueeze(0).repeat(4, 1, 1, 1)

img = None

def setup():
    global img
    size(W, H)
    img = create_image(W, H)
    flush()

def draw():
    t = frame_count * 0.02

    no_stroke()
    fill(255)
    circle(W / 2 + math.cos(t) * 150, H / 2 + math.sin(t) * 150, 60)

    flush()

    tensor = torch.as_tensor(cuda(), device="cuda")
    t_img = tensor.permute(2, 0, 1).unsqueeze(0).float()
    blurred = F.conv2d(t_img, BLUR, padding=KERNEL_SIZE // 2, groups=4)
    result = (blurred.squeeze(0).permute(1, 2, 0).clamp(0, 1) * 255).to(torch.uint8).contiguous()
    img.update_from(result)
    background(img)

run()
