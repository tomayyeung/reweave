import { Link, NavLink } from "react-router-dom";

import styles from "./NavBar.module.css";

export function NavBar() {
  return (
    <header className={styles.navbar}>
      <Link className={styles.brand} to="/">
        Reweave
      </Link>
      <nav className={styles.links} aria-label="Primary navigation">
        <NavLink
          className={({ isActive }) =>
            isActive ? `${styles.link} ${styles.active}` : styles.link
          }
          to="/create"
        >
          Create puzzle
        </NavLink>
      </nav>
      <div className={styles.futureLinks} aria-hidden="true" />
    </header>
  );
}
