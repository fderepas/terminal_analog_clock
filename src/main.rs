
use chrono::{Local, Timelike};
use ncurses::*;
use std::cmp::min;
use std::f64::consts::PI;
use std::env;
use std::path::PathBuf;

mod config_edit;

use config_edit::{Config};

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
    let mut start_at_0 = x_ori0 < x_ori1;
    if x_ori0 == x_ori1 {
        // the writing is vertical, write from top to bottom
        start_at_0 = y_ori0 < y_ori1
    }
    let mut x0 = if start_at_0 { x_ori0 } else { x_ori1 };
    let mut y0 = if start_at_0 { y_ori0 } else { y_ori1 };
    let x1 = if start_at_0 { x_ori1 } else { x_ori0 };
    let y1 = if start_at_0 { y_ori1 } else { y_ori0 };
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

/// Convert an angle (radians) into screen coordinates for an ellipse with
/// horizontal radius `a` and vertical radius `b`.
fn polar_to_cartesian_ellipse(cx: i32, cy: i32, angle: f64, a: f64, b: f64) -> (i32, i32) {
    // Y grows downwards on the terminal → we invert the Y component.
    let x = cx as f64 + a * angle.sin();
    let y = cy as f64 - b * angle.cos(); // minus = “up”
    (x.round() as i32, y.round() as i32)
}

fn restore_ncurses_context(cfg: &Config) {
    use_default_colors();
    cbreak();
    noecho();
    keypad(stdscr(), true);
    nodelay(stdscr(), true);
    curs_set(CURSOR_VISIBILITY::CURSOR_INVISIBLE);

    if has_colors() {
        start_color();
        let circle_color =cfg.get_option("circle color") as i16;
        let hours_color =cfg.get_option("hours color") as i16;
        let minutes_color =cfg.get_option("minutes color") as i16;
        let seconds_color =cfg.get_option("seconds color") as i16;
        let digits_color =cfg.get_option("digits color") as i16;
        
        init_pair(1, circle_color, -1); // ellipse
        init_pair(2, hours_color, -1); // hour hand
        init_pair(3, minutes_color, -1); // minute hand
        init_pair(4, seconds_color, -1); // second hand
        init_pair(5, digits_color, -1); // digits
    }
}

fn main() {
    let home = env::var("HOME").expect("Could not find HOME environment variable");

    // 2. Build the path safely
    let mut path = PathBuf::from(home);
    path.push(".tac.json");
    let mut cfg = Config::load(path.to_str().unwrap());


    // Init ncurses
    setlocale(LcCategory::all, "");
    initscr();
    start_color();
    restore_ncurses_context(&cfg);

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
        let a = 2 * b + (cfg.get_int("clock width") as i32); 

        // ----- clear screen -----
        erase();

        // ----- draw the ellipse (the “clock”) -----
        if cfg.get_option("clock border") /* user_config.show_circle */ == 1 {
            if has_colors() {
                attron(COLOR_PAIR(1));
            }
            draw_ellipse(cx, cy, a, b, '*' as chtype);
            if has_colors() {
                attroff(COLOR_PAIR(1));
            }
        } else if cfg.get_option("clock border") == 2 {
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
        } else if cfg.get_option("clock border") == 3 {
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
        let hour = (cfg.get_int("local time offset") + (now.hour() as i64)) % 12;
        let minute = now.minute();
        let second = match cfg.get_option("display seconds") /*user_config.show_seconds*/ {
            2 | 4 => now.second() * 1000 + (now.nanosecond() / 1_000_000),
            _ => now.second(),
        } as f64;

        // Angles: 0 rad = 12 o'clock, increase clockwise.
        let hour_angle = 2.0 * PI * ((hour as f64) + (minute as f64) / 60.0) / 12.0;
        let minute_angle = if cfg.get_bool("continuous minutes") /* user_config.continuous_minutes */ {
            2.0 * PI * ((minute as f64) + (second as f64) / 60.0) / 60.0
        } else {
            2.0 * PI * (minute as f64) / 60.0
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
            if cfg.get_int("numbers") /* user_config.show_numbers */ == 2 {
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
            } else if cfg.get_int("numbers") /* user_config.show_numbers*/ == 1 {
                draw_line(dx, dy, dx, dy, "*");
            }
        }

        // ----- second hand -----
        if cfg.get_option("display seconds") /* user_config.show_seconds*/ > 0 {
            let second_angle = match cfg.get_option("display seconds") {
                2 | 4 => 2.0 * PI * second / 60000.0,
                _ => 2.0 * PI * second / 60.0,
            };
            let (sx, sy) = polar_to_cartesian_ellipse(cx, cy, second_angle, a as f64, b as f64);
            if has_colors() {
                attron(COLOR_PAIR(4));
            }
            if cfg.get_option("display seconds") < 3 {
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
        if ch== 27 as i32 {
            cfg.terminal_edit_json();
            restore_ncurses_context(&cfg);
        }
        if ch == 'q' as i32 || ch == 'Q' as i32 {
            break;
        }
        if ch == 's' as i32 || ch == 'S' as i32 {
            cfg.set_option("display seconds",
                           ((cfg.get_option("display seconds") as i64) +1) % 5);
        }
        if ch == 'c' as i32 || ch == 'C' as i32 {
            cfg.set_option("clock border",
                           ((cfg.get_option("clock border") as i64) +1) % 4);
        }
        if ch == 'n' as i32 || ch == 'N' as i32 {
            cfg.set_option("numbers",
                           ((cfg.get_option("numbers") as i64) +1) % 3);
        }
        if ch == 'm' as i32 || ch == 'M' as i32 {
            cfg.set_bool("continuous minutes",
                         !cfg.get_bool("continuous minutes"));
        }
        if ch == '+' as i32 {
            if cfg.get_int("clock width")  < (b as i64) {
                cfg.set_int("clock width",cfg.get_int("clock width") -1);
            }
        }
        if ch == '-' as i32 {
            if cfg.get_int("clock width")  > (-b as i64) {
                cfg.set_int("clock width",cfg.get_int("clock width") -1);
            }
        }

        if cfg.get_option("display seconds") == 2 || cfg.get_option("display seconds") == 4 {
            // Sleep a little (≈30ms → ~33fps)
            napms(30);
        } else {
            // Sleep 1 second
            //napms(333);
            // Sleep .2 second
            napms(66);
            
        }
    }

    /* ---------- clean up ---------- */
    endwin();
}
