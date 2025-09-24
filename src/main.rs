/*
Colors are defined in curses.h:

#define COLOR_BLACK	0
#define COLOR_RED	1
#define COLOR_GREEN	2
#define COLOR_YELLOW	3
#define COLOR_BLUE	4
#define COLOR_MAGENTA	5
#define COLOR_CYAN	6
#define COLOR_WHITE	7
 */
use chrono::{Local, Timelike};
use ncurses::*;
use once_cell::sync::Lazy;
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use std::cmp::min;
use std::f64::consts::PI;
use std::fs::File;
use std::io::{Read, Write};

#[derive(Serialize, Clone, Deserialize, Debug)]
struct UserConfig {
    version: u32,
    color_background: i16,
    color_circle: i16,
    color_digits: i16,
    color_seconds: i16,
    color_minutes: i16,
    color_hours: i16,
    show_seconds: i16,
    show_circle: i16,
    show_numbers: i16,
    continuous_minutes: i16,
    delta_a: i16,
}

fn load_user_from_json(path: &str) -> std::io::Result<UserConfig> {
    let expanded = shellexpand::tilde(path).as_ref().to_owned();
    let mut file = File::open(expanded)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    // Deserialize the JSON string back into a UserConfig struct
    let user: UserConfig = serde_json::from_str(&contents)?;

    Ok(user)
}

/// Plot the four symmetric points of an ellipse.
fn plot_ellipse_points(cx: i32, cy: i32, x: i32, y: i32, ch: chtype) {
    // Quadrant symmetry
    let points = [
        (cx + x, cy + y),
        (cx - x, cy + y),
        (cx + x, cy - y),
        (cx - x, cy - y),
    ];
    for &(px, py) in &points {
        if px >= 0 && py >= 0 {
            mvaddch(py, px, ch);
        }
    }
}

/// Draw an ellipse centred at (cx,cy) with horizontal radius `a` and vertical radius `b`.
/// Uses the classic integer‑based midpoint ellipse algorithm.
fn draw_ellipse(cx: i32, cy: i32, a: i32, b: i32, ch: chtype) {
    // Squares of radii – keep them as i64 to avoid overflow in the integer part.
    let a2 = (a as i64) * (a as i64);
    let b2 = (b as i64) * (b as i64);

    // ---------- Region 1 (slope > –1) ----------
    let mut x: i32 = 0;
    let mut y: i32 = b;
    let mut d1: i64 = b2 - a2 * b as i64 + (a2 / 4);

    while (2 * b2 * (x as i64)) < (2 * a2 * (y as i64)) {
        plot_ellipse_points(cx, cy, x, y, ch);
        if d1 < 0 {
            d1 += 2 * b2 * (x as i64) + 3 * b2;
        } else {
            d1 += 2 * b2 * (x as i64) - 2 * a2 * (y as i64) + 3 * b2;
            y -= 1;
        }
        x += 1;
    }

    // ---------- Region 2 (slope ≤ –1) ----------
    // The classic formula uses a half‑pixel offset (x+0.5) and (y‑1).
    // We compute it with `f64` to keep the 0.5 without casting problems.
    let mut d2: f64 = b2 as f64 * ((x as f64) + 0.5).powi(2)
        + a2 as f64 * ((y as f64) - 1.0).powi(2)
        - (a2 * b2) as f64;

    while y >= 0 {
        plot_ellipse_points(cx, cy, x, y, ch);
        if d2 > 0.0 {
            d2 -= 2.0 * a2 as f64 * (y as f64) + 3.0 * a2 as f64;
        } else {
            d2 += 2.0 * b2 as f64 * (x as f64) - 2.0 * a2 as f64 * (y as f64) + 3.0 * a2 as f64;
            x += 1;
        }
        y -= 1;
    }
}

/// Bresenham line drawing – draws a straight line from (x0,y0) to (x1,y1)
/// using a repeating string pattern for the line's texture.
fn draw_line(x_ori0: i32, y_ori0: i32, x_ori1: i32, y_ori1: i32, pattern: &str) {
    // If the pattern is empty, there's nothing to draw.
    if pattern.is_empty() {
        return;
    }

    let mut x0 = if x_ori0 < x_ori1 { x_ori0 } else { x_ori1 };
    let mut y0 = if x_ori0 < x_ori1 { y_ori0 } else { y_ori1 };
    let mut x1 = if x_ori0 < x_ori1 { x_ori1 } else { x_ori0 };
    let mut y1 = if x_ori0 < x_ori1 { y_ori1 } else { y_ori0 };
    let dx = (x1 - x0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let dy = -(y1 - y0).abs();
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy; // error value

    // Create an iterator that cycles through the characters of the pattern indefinitely.
    let mut pattern_chars = pattern.chars().cycle();

    loop {
        // Get the next character from our cycling iterator and draw it.
        // .unwrap() is safe here because we checked that the pattern is not empty.
        let ch = pattern_chars.next().unwrap();
        mvaddch(y0, x0, ch as chtype);

        // Check for the end of the line
        if x0 == x1 && y0 == y1 {
            break;
        }

        // Bresenham's algorithm logic
        let e2 = 2 * err;
        if e2 >= dy {
            err += dy;
            x0 += sx;
        }
        if e2 <= dx {
            err += dx;
            y0 += sy;
        }
    }
}

/*
/// Bresenham line drawing – draws a straight line from (x0,y0) to (x1,y1)
fn draw_line(x0: i32, y0: i32, x1: i32, y1: i32, ch: chtype) {
    let mut x0 = x0;
    let mut y0 = y0;
    let dx = (x1 - x0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let dy = -(y1 - y0).abs();
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy; // error value

    loop {
        mvaddch(y0, x0, ch);
        if x0 == x1 && y0 == y1 {
            break;
        }
        let e2 = 2 * err;
        if e2 >= dy {
            err += dy;
            x0 += sx;
        }
        if e2 <= dx {
            err += dx;
            y0 += sy;
        }
    }
}
 */

/// Convert an angle (radians) into screen coordinates for an ellipse with
/// horizontal radius `a` and vertical radius `b`.
fn polar_to_cartesian_ellipse(cx: i32, cy: i32, angle: f64, a: f64, b: f64) -> (i32, i32) {
    // Y grows downwards on the terminal → we invert the Y component.
    let x = cx as f64 + a * angle.sin();
    let y = cy as f64 - b * angle.cos(); // minus = “up”
    (x.round() as i32, y.round() as i32)
}

const USER_CONFIG_FILE: &'static str = "~/.terminal_analog_clock.json";

static GLOBAL_USER_CONFIG: Lazy<Mutex<UserConfig>> = Lazy::new(|| {
    Mutex::new(UserConfig {
        version: 1,
        color_background: COLOR_BLACK,
        color_circle: COLOR_GREEN,
        color_digits: COLOR_WHITE,
        color_seconds: COLOR_CYAN,
        color_minutes: COLOR_YELLOW,
        color_hours: COLOR_RED,
        show_seconds: 1,
        show_circle: 1,
        show_numbers: 2,
        continuous_minutes: 0,
        delta_a: 0,
    })
});

fn save_config(
    user_config: &parking_lot::lock_api::MutexGuard<'_, parking_lot::RawMutex, UserConfig>,
) -> std::io::Result<()> {
    let ug: &UserConfig = &user_config;
    let json_string = serde_json::to_string_pretty(ug)?;
    let expanded = shellexpand::tilde(USER_CONFIG_FILE).as_ref().to_owned();
    let mut file = File::create(expanded)?;
    file.write_all(json_string.as_bytes())?;
    Ok(())
}

fn main() {
    // Set configuration
    let mut user_config = GLOBAL_USER_CONFIG.lock();
    match load_user_from_json(USER_CONFIG_FILE) {
        Ok(loaded_user) => {
            *user_config = loaded_user;
        }
        Err(_) => {
            // no existing config file, use defaut values
        }
    }
    let _ = save_config(&user_config);

    // Init ncurses
    initscr();
    start_color();
    use_default_colors();
    cbreak();
    noecho();
    keypad(stdscr(), true);
    nodelay(stdscr(), true);
    curs_set(CURSOR_VISIBILITY::CURSOR_INVISIBLE);

    if has_colors() {
        start_color();
        init_pair(1, user_config.color_circle, -1); // ellipse
        init_pair(2, user_config.color_hours, -1); // hour hand
        init_pair(3, user_config.color_minutes, -1); // minute hand
        init_pair(4, user_config.color_seconds, -1); // second hand
        init_pair(5, user_config.color_digits, -1); // digits
    }

    /* ---------- main loop ---------- */
    loop {
        // ----- terminal size & centre -----
        let mut rows = 0;
        let mut cols = 0;
        getmaxyx(stdscr(), &mut rows, &mut cols);
        let cx = cols / 2;
        let cy = rows / 2;

        // ----- choose radii so that width = 2 × height and everything fits -----
        // a = horizontal radius, b = vertical radius, and a = 2·b.
        // Must satisfy: a <= cols/2‑1  and  b <= rows/2‑1.
        // Hence: b <= min(rows/2‑1, (cols/2‑1)/2)
        let max_b = min(rows / 2 - 1, (cols / 2 - 1) / 2);
        let b = max_b; // vertical radius (the “height” of the clock)
                       //        let a = b;          // horizontal radius (twice the height)
                       // horizontal radius = (twice the height) + custom offset
        let a = 2 * b + (user_config.delta_a as i32);

        // ----- clear screen -----
        erase();

        // ----- draw the ellipse (the “clock”) -----
        if user_config.show_circle == 1 {
            if has_colors() {
                attron(COLOR_PAIR(1));
            }
            draw_ellipse(cx, cy, a, b, '*' as chtype);
            if has_colors() {
                attroff(COLOR_PAIR(1));
            }
        } else if user_config.show_circle == 2 {
            if has_colors() {
                attron(COLOR_PAIR(1));
            }
            for i in 0..60 {
                let (dx, dy) = polar_to_cartesian_ellipse(
                    cx,
                    cy,
                    2.0 * PI * (i as f64) / 60.0,
                    a as f64,
                    b as f64,
                );
                if i % 5 == 0 {
                    let (ddx, ddy) = polar_to_cartesian_ellipse(
                        cx,
                        cy,
                        2.0 * PI * (i as f64) / 60.0,
                        (a as f64) * 0.95,
                        (b as f64) * 0.95,
                    );
                    draw_line(dx, dy, ddx, ddy, "*");
                } else {
                    draw_line(dx, dy, dx, dy, ".");
                }
            }
            if has_colors() {
                attroff(COLOR_PAIR(1));
            }
        } else if user_config.show_circle == 3 {
            if has_colors() {
                attron(COLOR_PAIR(1));
            }
            for i in 0..12 {
                let (dx, dy) = polar_to_cartesian_ellipse(
                    cx,
                    cy,
                    2.0 * PI * (i as f64) / 12.0,
                    a as f64,
                    b as f64,
                );
                draw_line(dx, dy, dx, dy, "*");
            }
            if has_colors() {
                attroff(COLOR_PAIR(1));
            }
        }

        // ----- current local time -----
        let now = Local::now();
        let hour = now.hour() % 12;
        let minute = now.minute();
        let second = match user_config.show_seconds {
            2 | 4 => now.second() * 1000 + (now.nanosecond() / 1_000_000),
            _ => now.second(),
        } as f64;

        // Angles: 0 rad = 12 o'clock, increase clockwise.
        let hour_angle = 2.0 * PI * ((hour as f64) + (minute as f64) / 60.0) / 12.0;
        let minute_angle = match user_config.continuous_minutes {
            0 => 2.0 * PI * (minute as f64) / 60.0,
            _ => 2.0 * PI * ((minute as f64) + (second as f64) / 60.0) / 60.0,
        };

        for i in 1..13 {
            if has_colors() {
                attron(COLOR_PAIR(5));
            }
            let (dx, dy) = polar_to_cartesian_ellipse(
                cx,
                cy,
                2.0 * PI * (i as f64) / 12.0,
                (a as f64) * 0.9,
                (b as f64) * 0.9,
            );
            if user_config.show_numbers == 2 {
                if i > 9 {
                    draw_line(dx - 1, dy, dx, dy, "1");
                }
                let s =  (i % 10).to_string();
                draw_line(
                    dx,
                    dy,
                    dx,
                    dy,
                    &s,
                );
            } else if user_config.show_numbers == 1 {
                draw_line(dx, dy, dx, dy, "*");
            }
        }

        // ----- second hand -----
        if user_config.show_seconds > 0 {
            let second_angle = match user_config.show_seconds {
                2 | 4 => 2.0 * PI * second / 60000.0,
                _ => 2.0 * PI * second / 60.0,
            };
            let (sx, sy) = polar_to_cartesian_ellipse(cx, cy, second_angle, a as f64, b as f64);
            if has_colors() {
                attron(COLOR_PAIR(4));
            }
            if user_config.show_seconds < 3 {
                draw_line(cx, cy, sx, sy, ".");
            } else {
                let (bx, by) = polar_to_cartesian_ellipse(
                    cx,
                    cy,
                    second_angle,
                    (a as f64) * 0.8,
                    (b as f64) * 0.8,
                );
                draw_line(bx, by, sx, sy, ".");
            }
            if has_colors() {
                attroff(COLOR_PAIR(4));
            }
        }
        // ----- minute hand -----
        let (mx, my) =
            polar_to_cartesian_ellipse(cx, cy, minute_angle, (a as f64) * 0.9, (b as f64) * 0.9);
        if has_colors() {
            attron(COLOR_PAIR(3));
        }
        draw_line(
            cx + (cx - mx) / 10,
            cy + (cy - my) / 10,
            mx,
            my,
            "minutes",
        );
        if has_colors() {
            attroff(COLOR_PAIR(3));
        }
        // ----- hour hand -----
        let (hx, hy) =
            polar_to_cartesian_ellipse(cx, cy, hour_angle, (a as f64) * 0.7, (b as f64) * 0.7);
        if has_colors() {
            attron(COLOR_PAIR(2));
        }
        draw_line(
            cx + (cx - hx) / 10,
            cy + (cy - hy) / 10,
            hx,
            hy,
            "HOURS",
        );
        if has_colors() {
            attroff(COLOR_PAIR(2));
        }

        // ----- refresh & input -----
        refresh();

        // quit on 'q' or 'Q'
        let ch = getch();
        if ch == 'q' as i32 || ch == 'Q' as i32 {
            break;
        }
        if ch == 's' as i32 || ch == 'S' as i32 {
            user_config.show_seconds += 1;
            if user_config.show_seconds > 4 {
                user_config.show_seconds %= 5;
            }
            let _ = save_config(&user_config);
        }
        if ch == 'c' as i32 || ch == 'C' as i32 {
            user_config.show_circle += 1;
            if user_config.show_circle > 3 {
                user_config.show_circle %= 4;
            }
            let _ = save_config(&user_config);
        }
        if ch == 'n' as i32 || ch == 'N' as i32 {
            user_config.show_numbers += 1;
            if user_config.show_numbers > 2 {
                user_config.show_numbers %= 3;
            }
            let _ = save_config(&user_config);
        }
        if ch == 'm' as i32 || ch == 'M' as i32 {
            user_config.continuous_minutes += 1;
            if user_config.continuous_minutes > 1 {
                user_config.continuous_minutes %= 2;
            }
            let _ = save_config(&user_config);
        }
        if ch == '+' as i32 {
            if (user_config.delta_a as i32) < b {
                user_config.delta_a += 1;
            }
            let _ = save_config(&user_config);
        }
        if ch == '-' as i32 {
            if (user_config.delta_a as i32) > -b {
                user_config.delta_a -= 1;
            }
            let _ = save_config(&user_config);
        }

        if user_config.show_seconds == 2 || user_config.show_seconds == 4 {
            // Sleep a little (≈30ms → ~33fps)
            napms(30);
        } else {
            // Sleep 1 second
            napms(333);
        }
    }

    /* ---------- clean up ---------- */
    endwin();
}
