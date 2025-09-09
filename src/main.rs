use chrono::{Local, Timelike};
use ncurses::*;
use std::cmp::min;
use std::f64::consts::PI;

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

    // ---------- Region 1 (slope > –1) ----------
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

    // ---------- Region 2 (slope ≤ –1) ----------
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

/// Convert an angle (radians) into screen coordinates for an ellipse with
/// horizontal radius `a` and vertical radius `b`.
fn polar_to_cartesian_ellipse(cx: i32, cy: i32, angle: f64, a: f64, b: f64) -> (i32, i32) {
    // Y grows downwards on the terminal → we invert the Y component.
    let x = cx as f64 + a * angle.sin();
    let y = cy as f64 - b * angle.cos(); // minus = “up”
    (x.round() as i32, y.round() as i32)
}

fn main() {
    let mut show_seconds = 1;
    let mut show_circle = 1;
    let mut show_numbers = 2;
    /* ---------- ncurses initialisation ---------- */
    initscr();
    cbreak();
    noecho();
    keypad(stdscr(), true);
    nodelay(stdscr(), true);
    curs_set(CURSOR_VISIBILITY::CURSOR_INVISIBLE);

    if has_colors() {
        start_color();
        init_pair(1, COLOR_GREEN, COLOR_BLACK); // ellipse
        init_pair(2, COLOR_RED, COLOR_BLACK); // hour hand
        init_pair(3, COLOR_YELLOW, COLOR_BLACK); // minute hand
        init_pair(4, COLOR_CYAN, COLOR_BLACK); // second hand
        init_pair(5, COLOR_WHITE, COLOR_BLACK); // digits
    }

    /* ---------- main loop ---------- */
    loop {
        // ----- terminal size & centre -----
        let mut rows = 0;
        let mut cols = 0;
        getmaxyx(stdscr(), &mut rows, &mut cols);
        let cx = cols / 2;
        let cy = rows / 2;

        // ----- choose radii so that width = 2 × height and everything fits -----
        // a = horizontal radius, b = vertical radius, and a = 2·b.
        // Must satisfy: a <= cols/2‑1  and  b <= rows/2‑1.
        // Hence: b <= min(rows/2‑1, (cols/2‑1)/2)
        let max_b = min(rows / 2 - 1, (cols / 2 - 1) / 2);
        let b = max_b; // vertical radius (the “height” of the clock)
                       //        let a = b;          // horizontal radius (twice the height)
        let a = 2 * b; // horizontal radius (twice the height)

        // ----- clear screen -----
        erase();

        // ----- draw the ellipse (the “clock”) -----
        if show_circle == 1 {
            if has_colors() {
                attron(COLOR_PAIR(1));
            }
            draw_ellipse(cx, cy, a, b, '*' as chtype);
            if has_colors() {
                attroff(COLOR_PAIR(1));
            }
        } else if show_circle == 2 {
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
                if i%5 == 0 {
                    draw_line(
                        dx,
                        dy,
                        dx,
                        dy,
                        '*' as chtype,
                    );
                } else {
                    draw_line(
                        dx,
                        dy,
                        dx,
                        dy,
                        '.' as chtype,
                    );
                }
            }
            if has_colors() {
                attroff(COLOR_PAIR(1));
            }
        }

        // ----- current local time -----
        let now = Local::now();
        let hour = now.hour() % 12;
        let minute = now.minute();

        // Angles: 0 rad = 12 o’clock, increase clockwise.
        let hour_angle = 2.0 * PI * ((hour as f64) + (minute as f64) / 60.0) / 12.0;
        let minute_angle = 2.0 * PI * (minute as f64) / 60.0;

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
            if show_numbers == 2 {
                if i > 9 {
                    draw_line(dx - 1, dy, dx, dy, '1' as chtype);
                }
                draw_line(
                    dx,
                    dy,
                    dx,
                    dy,
                    std::char::from_digit(i % 10, 10).unwrap() as chtype,
                );
            } else if show_numbers == 1 {
                draw_line(dx, dy, dx, dy, '*' as chtype);
            }
        }

        // ----- second hand -----
        if show_seconds > 0 {
            let second = match show_seconds {
                1 => now.second(),
                _ => now.second() * 1000 + (now.nanosecond() / 1_000_000),
            } as f64;
            let second_angle = match show_seconds {
                1 => 2.0 * PI * second / 60.0,
                _ => 2.0 * PI * second / 60000.0,
            };
            let (sx, sy) = polar_to_cartesian_ellipse(cx, cy, second_angle, a as f64, b as f64);
            if has_colors() {
                attron(COLOR_PAIR(4));
            }
            draw_line(cx, cy, sx, sy, '.' as chtype);
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
        draw_line(cx, cy, mx, my, 'M' as chtype);
        if has_colors() {
            attroff(COLOR_PAIR(3));
        }
        // ----- hour hand -----
        let (hx, hy) =
            polar_to_cartesian_ellipse(cx, cy, hour_angle, (a as f64) * 0.7, (b as f64) * 0.7);
        if has_colors() {
            attron(COLOR_PAIR(2));
        }
        draw_line(cx, cy, hx, hy, 'H' as chtype);
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
            show_seconds += 1;
            if show_seconds > 2 {
                show_seconds %= 3;
            }
        }
        if ch == 'c' as i32 || ch == 'C' as i32 {
            show_circle += 1;
            if show_circle > 2 {
                show_circle %= 3;
            }
        }
        if ch == 'n' as i32 || ch == 'N' as i32 {
            show_numbers += 1;
            if show_numbers > 2 {
                show_numbers %= 3;
            }
        }

        // Sleep a little (≈30 ms → ~33 fps)
        napms(30);
    }

    /* ---------- clean up ---------- */
    endwin();
}
