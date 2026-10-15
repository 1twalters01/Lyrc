defmodule SessionsTest do
  use ExUnit.Case
  doctest Sessions

  test "greets the world" do
    assert Sessions.hello() == :world
  end
end
