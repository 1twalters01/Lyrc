defmodule ResourcesTest do
  use ExUnit.Case
  doctest Resources

  test "greets the world" do
    assert Resources.hello() == :world
  end
end
