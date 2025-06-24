-- migrations/xxxx_create_notes_table.sql

CREATE TABLE notes (
    id SERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL,         -- 关联用户
    title VARCHAR(255) NOT NULL,
    content TEXT NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT now(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT now()
);

-- 可以根据需要加外键约束，如 user_id 关联 users 表的 id
ALTER TABLE notes
  ADD CONSTRAINT fk_user
  FOREIGN KEY (user_id) REFERENCES users(id);
