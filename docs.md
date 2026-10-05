
![alt text](image.png)

Ugoku is an experimental simulation tool for building and animating(soon) mechanical linkages. Largely inspired by OpenSCAD, but with movement in mind, it lets you define structures via scripts and simulate their motion using position-based dynamics.



---
## Program Structure
### `sim`
```ugoku
sim identifier { ... }
```

Holds the simulation (All joints, links, and constraints)

**Arguments:**
- `identifier` - Name of the simulation

**Example:**

```ugoku
sim system {
    joint a (0, 0)
    joint b (0,2)
    link c(a,b)
    // ... other statements
}
```
---

### `joint`
```ugoku
joint identifier (x, y)
joint identifier (x, y, z)
```

Declares a joint at a specific position in 2D or 3D.

**Arguments:**
- `identifier` - Joint name
- `x` - X coordinate
- `y` - Y coordinate  
- `z` - Z coordinate (Needed for 3d)

**Example:**
```ugoku
joint origin (0, 0)
joint pivot (10, 20, 5)
```

---


### `link`
```ugoku
link identifier (joint1, joint2)
```

Declares a connection between two joints.

**Arguments:**
- `identifier` - Link name
- `joint1` - First joint name
- `joint2` - Second joint name

**Example:**
```ugoku
link arm_segment (shoulder, elbow)
```

---

## Constraints


### `distance`
```ugoku
distance (joint1, joint2, distance)
```

Maintains a fixed distance between two joints.

**Arguments:**
- `joint1` - First joint name
- `joint2` - Second joint name
- `distance` - Fixed distance value

**Example:**
```ugoku
distance (joint_a, joint_b, 10.5)
```
---
### `fixed`
```ugoku
fixed (joint1, joint2, ...)
```

Prevents specified joints from moving (locks them in place).

**Arguments:**
- `joint1, joint2, ...` - One or more joint identifiers

**Example:**
```ugoku
fixed (base)
fixed (anchor1, anchor2, anchor3)
```
---
### `plane`
```ugoku
plane ((joint1, joint2, ...), axis)
```

Projects a joint onto a specific plane.

**Arguments:**
- `(joint1, joint2, ...)` - List of joint names in parentheses
- `axis` - Plane normal direction (`X`, `Y`, or `Z`)

**Example:**
```ugoku
plane ((joint1, joint2), Y)
```
---
### `prismatic_vector`
```ugoku
prismatic_vector (joint1, joint2, ..., axis, origin)
```

Allows joints to slide along an arbitrary vector direction.

**Arguments:**
- `joint1, joint2, ...` - Joint names to constrain
- `axis` - Direction of movement (`X`, `Y`, or `Z`)
- `origin` - Origin point as `(x, y, z)`

**Example:**
```ugoku
prismatic_vector (slider1, slider2, Z, (0, 0, 0))
```
---
### `prismatic_link`
```ugoku
prismatic_link ((joint1, joint2, ...), link_id, (origin))
```

Allows joints to slide along a link direction.

**Arguments:**
- `(joint1, joint2, ...)` - List of joint names in parentheses
- `link_id` - Reference link name
- `(origin)` - Origin point as `(x, y, z)` in parentheses

**Example:**
```ugoku
prismatic_link ((piston), guide_rail, (5, 0, 0))
```
---
### `fixed_angle`
```ugoku
fixed_angle (link1, link2, pivot_joint, angle)
```

Maintains a fixed angle between two links at a pivot joint.

**Arguments:**
- `link1` - First link name
- `link2` - Second link name
- `pivot_joint` - Joint where links meet
- `angle` - Fixed angle value (with optional unit - radians/degeress)

**Example:**
```ugoku
fixed_angle (upper_arm, lower_arm, elbow, 90degrees)
```

**Note**
Units can either be deg/rad or degrees/radians

### `revolute`
```ugoku
revolute (pivot_joint, moving_joint, axis, min_angle, max_angle)
```

Creates a rotational joint with angle limits.

**Arguments:**
- `pivot_joint` - Fixed rotation point
- `moving_joint` - Joint that rotates around pivot
- `axis` - Rotation axis as `(x, y, z)`
- `min_angle` - Minimum angle limit
- `max_angle` - Maximum angle limit

**Example:**
```ugoku
revolute (shoulder, upper_arm, (0, 0, 1), -90 deg, 90 deg)
```

---

## Data Types

### `identifier`
```
[A-Za-z][A-Za-z0-9_]*
```

Alphanumeric string starting with a letter, may contain underscores.

### `number`
```
-?[0-9]+(\.[0-9]+)?
```

Integer or floating-point number, optionally negative.

### `vec3`
```
(x, y, z)
```

Three-dimensional vector with numeric components.

### `axis`
```
X | Y | Z
```

Single character axis specifier.

### `angle_value`
```
number (degrees|radians|deg|rad)?
```

Numeric angle with optional unit. Defaults to radians if no unit specified.

**Examples:**
- `90 degrees`
- `1.57 radians` 
- `45 deg`
- `3.14` (radians)

---

## Complete Example

```ugoku
sim arm {
    joint base (0, 0, 0)
    joint shoulder (0, 0, 10)
    joint elbow (20, 0, 10)
    joint wrist (35, 0, 10)
    
    link base_to_shoulder (base, shoulder)
    link upper_arm (shoulder, elbow)
    link forearm (elbow, wrist)
    
    fixed (base)
    distance (shoulder, elbow, 20)
    distance (elbow, wrist, 15)
    
    revolute (shoulder, elbow, (0, 0, 1), -180 deg, 180 deg)
    revolute (elbow, wrist, (0, 0, 1), -90 deg, 90 deg)
}
