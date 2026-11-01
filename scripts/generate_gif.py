import os
import re
import shutil
import subprocess
import sys
from PIL import Image, ImageDraw, ImageFont

ANSI_RE = re.compile(r'\x1b\[([0-9;]*)m')

FONT_PATH = "C:/Windows/Fonts/consola.ttf"
FONT_SIZE = 15
LINE_HEIGHT = 21
WIN_W = 860
WIN_H = 500

COLOR_BG = (24, 24, 37)          # Mocha mantle
COLOR_TERM_BG = (30, 30, 46)     # Mocha base
COLOR_BORDER = (49, 50, 68)      # Mocha surface0
COLOR_TEXT_DEF = (205, 214, 244) # Mocha text
COLOR_PROMPT_DIR = (137, 180, 250) # Blue
COLOR_PROMPT_SYM = (203, 166, 247) # Mauve
COLOR_CMD = (245, 224, 220)      # Rosewater
COLOR_TITLE = (147, 153, 178)    # Overlay0

def parse_ansi_line(line):
    pos = 0
    runs = []
    current_fg = COLOR_TEXT_DEF
    current_bg = None

    for m in ANSI_RE.finditer(line):
        text = line[pos:m.start()]
        if text:
            runs.append((text, current_fg, current_bg))
        pos = m.end()

        raw_args = m.group(1)
        if not raw_args:
            current_fg = COLOR_TEXT_DEF
            current_bg = None
            continue

        args = [int(x) for x in raw_args.split(';') if x]
        if not args or args == [0]:
            current_fg = COLOR_TEXT_DEF
            current_bg = None
        elif args[0] == 38 and len(args) >= 5 and args[1] == 2:
            current_fg = (args[2], args[3], args[4])
        elif args[0] == 48 and len(args) >= 5 and args[1] == 2:
            current_bg = (args[2], args[3], args[4])
        elif 90 in args:
            current_fg = (127, 132, 156)
        elif 32 in args:
            current_fg = (166, 227, 161)
        elif 33 in args:
            current_fg = (249, 226, 175)
        elif 34 in args:
            current_fg = (137, 180, 250)
        elif 36 in args:
            current_fg = (148, 226, 213)
        elif 31 in args:
            current_fg = (243, 139, 168)

    text = line[pos:]
    if text:
        runs.append((text, current_fg, current_bg))
    return runs

def draw_window_frame(draw, font_small):
    # Terminal Window Shadow & Background
    term_x0, term_y0 = 15, 15
    term_x1, term_y1 = WIN_W - 15, WIN_H - 15
    radius = 12

    # Draw rounded terminal background
    draw.rounded_rectangle([term_x0, term_y0, term_x1, term_y1], radius=radius, fill=COLOR_TERM_BG, outline=COLOR_BORDER, width=1)

    # Titlebar background
    draw.rounded_rectangle([term_x0, term_y0, term_x1, term_y0 + 36], radius=radius, fill=(24, 24, 37))
    draw.rectangle([term_x0, term_y0 + 24, term_x1, term_y0 + 36], fill=(24, 24, 37))
    draw.line([term_x0, term_y0 + 36, term_x1, term_y0 + 36], fill=COLOR_BORDER, width=1)

    # Traffic light window buttons
    btn_y = term_y0 + 18
    draw.ellipse([term_x0 + 16, btn_y - 6, term_x0 + 28, btn_y + 6], fill=(243, 139, 168)) # Red
    draw.ellipse([term_x0 + 36, btn_y - 6, term_x0 + 48, btn_y + 6], fill=(249, 226, 175)) # Yellow
    draw.ellipse([term_x0 + 56, btn_y - 6, term_x0 + 68, btn_y + 6], fill=(166, 227, 161)) # Green

    # Window title
    title = "rcat — bash"
    bbox = font_small.getbbox(title)
    tw = bbox[2] - bbox[0]
    draw.text(((WIN_W - tw) // 2, term_y0 + 10), title, font=font_small, fill=COLOR_TITLE)

def render_frame(font, font_small, prompt_cmd, output_lines=None, show_cursor=True):
    img = Image.new("RGB", (WIN_W, WIN_H), COLOR_BG)
    draw = ImageDraw.Draw(img)

    draw_window_frame(draw, font_small)

    cur_x = 32
    cur_y = 62

    # Draw prompt
    prompt_dir = "~/rcat "
    prompt_sym = "❯ "
    draw.text((cur_x, cur_y), prompt_dir, font=font, fill=COLOR_PROMPT_DIR)
    bbox = font.getbbox(prompt_dir)
    cur_x += bbox[2] - bbox[0]

    draw.text((cur_x, cur_y), prompt_sym, font=font, fill=COLOR_PROMPT_SYM)
    bbox = font.getbbox(prompt_sym)
    cur_x += bbox[2] - bbox[0]

    # Draw command typed so far
    draw.text((cur_x, cur_y), prompt_cmd, font=font, fill=COLOR_CMD)
    bbox = font.getbbox(prompt_cmd)
    cur_x += bbox[2] - bbox[0]

    if show_cursor:
        draw.text((cur_x + 2, cur_y), "▌", font=font, fill=COLOR_CMD)

    # Draw output lines if command has executed
    if output_lines:
        cur_y += LINE_HEIGHT + 4
        for raw_line in output_lines:
            if cur_y + LINE_HEIGHT > WIN_H - 25:
                break
            lx = 32
            parsed_runs = parse_ansi_line(raw_line)
            for text, fg, bg in parsed_runs:
                if bg:
                    bbox = font.getbbox(text)
                    w = bbox[2] - bbox[0]
                    draw.rectangle([lx, cur_y, lx + w, cur_y + LINE_HEIGHT - 2], fill=bg)
                draw.text((lx, cur_y), text, font=font, fill=fg)
                bbox = font.getbbox(text)
                lx += bbox[2] - bbox[0]
            cur_y += LINE_HEIGHT

    return img

def main():
    os.makedirs("target/frames", exist_ok=True)
    font = ImageFont.truetype(FONT_PATH, FONT_SIZE)
    font_small = ImageFont.truetype(FONT_PATH, 13)

    # 1. Capture rcat command outputs
    cmd1_out = subprocess.run(['target/debug/rcat.exe', 'Cargo.toml', '--style', 'full'], capture_output=True, text=True, encoding='utf-8').stdout.splitlines()
    cmd2_out = subprocess.run(['target/debug/rcat.exe', '-n', '-r', '1:16', 'src/main.rs'], capture_output=True, text=True, encoding='utf-8').stdout.splitlines()
    cmd3_out = subprocess.run(['target/debug/rcat.exe', '-X', 'Cargo.toml'], capture_output=True, text=True, encoding='utf-8').stdout.splitlines()[:16]

    scenes = [
        ("rcat --style full Cargo.toml", cmd1_out, 45),
        ("rcat -n -r 1:16 src/main.rs", cmd2_out, 45),
        ("rcat -X Cargo.toml", cmd3_out, 50),
    ]

    frame_idx = 0

    for cmd, out_lines, hold_count in scenes:
        # Initial prompt with blinking cursor
        for _ in range(6):
            img = render_frame(font, font_small, "", None, show_cursor=True)
            img.save(f"target/frames/frame_{frame_idx:04d}.png")
            frame_idx += 1

        # Typing animation
        for i in range(1, len(cmd) + 1):
            sub_cmd = cmd[:i]
            img = render_frame(font, font_small, sub_cmd, None, show_cursor=True)
            img.save(f"target/frames/frame_{frame_idx:04d}.png")
            frame_idx += 1

        # Press Enter & Show output
        for i in range(hold_count):
            img = render_frame(font, font_small, cmd, out_lines, show_cursor=False)
            img.save(f"target/frames/frame_{frame_idx:04d}.png")
            frame_idx += 1

    print(f"Total frames generated: {frame_idx}")

    # Generate palette and animated GIF using ffmpeg
    palette_cmd = [
        'ffmpeg', '-y', '-framerate', '15',
        '-i', 'target/frames/frame_%04d.png',
        '-vf', 'palettegen=stats_mode=diff',
        'target/palette.png'
    ]
    subprocess.run(palette_cmd, check=True)

    gif_cmd = [
        'ffmpeg', '-y', '-framerate', '15',
        '-i', 'target/frames/frame_%04d.png',
        '-i', 'target/palette.png',
        '-lavfi', 'paletteuse=dither=bayer:bayer_scale=5:diff_mode=rectangle',
        'assets/demo.gif'
    ]
    subprocess.run(gif_cmd, check=True)
    print("assets/demo.gif successfully created!")

if __name__ == '__main__':
    main()
